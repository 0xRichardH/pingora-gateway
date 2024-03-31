use std::collections::HashMap;

use pingora::server::Server;
use pingora_gateway::services::{proxy_service_tls, HostConfig};

fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::init();
}

fn add_tcp_proxy(server: &mut Server) {
    let host_configs = HashMap::from([
        (
            String::from("one.one.one.one"),
            HostConfig {
                proxy_addr: String::from("1.1.1.1:443"),
                proxy_tls: true,
                proxy_hostname: "one.one.one.one".to_string(),
                cert_path: format!("{}/keys/one.one.one.one.pem", env!("CARGO_MANIFEST_DIR")),
                key_path: format!(
                    "{}/keys/one.one.one.one-key.pem",
                    env!("CARGO_MANIFEST_DIR")
                ),
            },
        ),
        (
            String::from("one.one.one.two"),
            HostConfig {
                proxy_addr: String::from("1.0.0.1:443"),
                proxy_tls: true,
                proxy_hostname: "one.one.one.two".to_string(),
                cert_path: format!("{}/keys/one.one.one.two.pem", env!("CARGO_MANIFEST_DIR")),
                key_path: format!(
                    "{}/keys/one.one.one.two-key.pem",
                    env!("CARGO_MANIFEST_DIR")
                ),
            },
        ),
    ]);
    let proxy = proxy_service_tls(&server.configuration, "0.0.0.0:8999", host_configs);
    server.add_service(proxy);
}

fn main() {
    init_logger();

    //TODO: read command line arguments
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    add_tcp_proxy(&mut server);

    server.run_forever();
}
