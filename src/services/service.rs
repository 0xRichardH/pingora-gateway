use std::sync::Arc;

use async_trait::async_trait;
use log::debug;
use pingora::{
    listeners::{TlsAccept, tls::TlsSettings},
    proxy::http_proxy_service,
    server::configuration::ServerConf,
    tls::{self, ssl},
};

use super::{HostConfig, HostConfigs, proxy::ProxyService};

struct Callback(Vec<(String, tls::x509::X509, tls::pkey::PKey<tls::pkey::Private>)>);

impl Callback {
    fn new(config: HostConfigs) -> Self {
        let config = config
            .into_iter()
            .map(
                |(
                    _,
                    HostConfig {
                        proxy_hostname,
                        cert_path,
                        key_path,
                        proxy_addr: _,
                        proxy_tls: _,
                        filters: _,
                    },
                )| {
                    let cert_bytes = std::fs::read(cert_path).unwrap();
                    let cert = tls::x509::X509::from_pem(&cert_bytes).unwrap();

                    let key_bytes = std::fs::read(key_path).unwrap();
                    let key = tls::pkey::PKey::private_key_from_pem(&key_bytes).unwrap();

                    (proxy_hostname, cert, key)
                },
            )
            .collect();
        Self(config)
    }
}

#[async_trait]
impl TlsAccept for Callback {
    async fn certificate_callback(&self, ssl: &mut ssl::SslRef) -> () {
        let sni_provided = ssl.servername(ssl::NameType::HOST_NAME).unwrap();
        debug!("SNI provided: {}", sni_provided);
        let (_, cert, key) = self.0.iter().find(|x| x.0 == sni_provided).unwrap();
        tls::ext::ssl_use_certificate(ssl, cert).unwrap();
        tls::ext::ssl_use_private_key(ssl, key).unwrap();
    }
}

pub fn proxy_service_tls(
    server_conf: &Arc<ServerConf>,
    listen_addr: &str,
    host_configs: HostConfigs,
    root_cert_path: Option<String>,
) -> impl pingora::services::Service + use<> {
    // FIXME: we might need to remove use<> in the future
    let proxy_service = ProxyService::new(host_configs.clone());
    let mut service = http_proxy_service(server_conf, proxy_service);

    let cb = Callback::new(host_configs);
    let cb = Box::new(cb);
    let mut tls_settings = TlsSettings::with_callbacks(cb).unwrap();
    if let Some(root_cert_path) = root_cert_path {
        // load trusted root certificates
        tls_settings.set_ca_file(root_cert_path).unwrap();
    }
    service.add_tls_with_settings(listen_addr, None, tls_settings);

    service
}
