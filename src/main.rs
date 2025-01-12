#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use pingora::server::{configuration::Opt, Server};
use pingora_gateway::{
    config,
    prelude::*,
    services::{proxy_service_tls, HostConfig, HostConfigs},
};
use clap::Parser;

fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::init();
}

fn add_tcp_proxy(server: &mut Server, cfg: &config::ProxyService) {
    let host_configs = cfg
        .host_configs
        .iter()
        .fold(HostConfigs::new(), |mut host_config, c| {
            host_config.insert(
                c.proxy_hostname.clone(),
                HostConfig {
                    proxy_addr: c.proxy_addr.clone(),
                    proxy_tls: c.proxy_tls,
                    proxy_hostname: c.proxy_hostname.clone(),
                    cert_path: c.cert_path.clone(),
                    key_path: c.key_path.clone(),
                    filters: c.get_filters(),
                },
            );
            host_config
        });

    let proxy = proxy_service_tls(
        &server.configuration,
        &cfg.listen_addr,
        host_configs,
        cfg.root_cert_path.clone(),
    );
    server.add_service(proxy);
}

fn main() -> Result<()> {
    init_logger();

    let default_config_path = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let config = config::load_config(&default_config_path)?;

    let opt = Some(Opt::from_args());
    let mut server = Server::new(opt)?;
    server.bootstrap();

    add_tcp_proxy(&mut server, &config.proxy_service);

    server.run_forever()
}
