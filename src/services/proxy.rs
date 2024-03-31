use std::collections::HashMap;

use ::http::{HeaderName, StatusCode};
use async_trait::async_trait;
use log::{error, info};
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use pingora::proxy::ProxyHttp;

#[derive(Clone)]
pub struct HostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_hostname: String,
    pub cert_path: String,
    pub key_path: String,
}

type HostName = String;

pub struct ProxyService {
    host_configs: HashMap<HostName, HostConfig>,
}

pub struct ProxyCtx {
    host_config: Option<HostConfig>,
}

impl ProxyService {
    pub fn new(host_configs: HashMap<HostName, HostConfig>) -> Self {
        Self { host_configs }
    }

    fn check_ws_path(&self, path: &str) -> bool {
        // TODO: check if the path is a valid websocket path
        path.starts_with("/ws/")
    }
}

#[async_trait]
impl ProxyHttp for ProxyService {
    type CTX = ProxyCtx;

    fn new_ctx(&self) -> Self::CTX {
        ProxyCtx { host_config: None }
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let host_name = get_host_name(session);
        info!("Host name: {}", host_name);

        let Some(config) = self.host_configs.get(host_name.as_str()) else {
            error!("No proxy configuration found for host: {}", host_name);
            session
                .respond_error(StatusCode::BAD_REQUEST.as_u16())
                .await;

            return Ok(true);
        };
        ctx.host_config = Some(config.clone());

        let request_path = session.req_header().uri.path();
        if request_path == "/" {
            let mut resp_header = ResponseHeader::build(StatusCode::OK, None)?;
            resp_header.insert_header("Server", "Cloudflare")?;
            session.set_keepalive(None);
            session.write_response_header_ref(&resp_header).await?;
            session.write_response_body("Connecting...".into()).await?;

            // true: early return as the response is already written
            return Ok(true);
        }

        if !self.check_ws_path(request_path) {
            session.respond_error(404).await;

            // true: early return as the response is already written
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let Some(config) = ctx.host_config.as_ref() else {
            return Err(Error::new(ErrorType::InternalError));
        };

        let peer = Box::new(HttpPeer::new(
            config.proxy_addr.as_str(),
            config.proxy_tls,
            config.proxy_hostname.clone(),
        ));
        Ok(peer)
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // replace existing header if any
        upstream_response.insert_header("Server", "Cloudflare")?;

        // because pingora doesn't support h3
        upstream_response.remove_header("alt-svc");

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());

        info!(
            "{} response code: {response_code}",
            self.request_summary(session, ctx)
        );
    }
}

fn get_host_name(session: &Session) -> String {
    let header = session
        .get_header(HeaderName::from_static("host"))
        .map(|v| v.to_str());
    if let Some(Ok(host)) = header {
        host.to_string()
    } else {
        String::new()
    }
}
