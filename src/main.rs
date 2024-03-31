use pingora::{proxy::http_proxy_service, server::Server};
use pingora_gateway::services::v2ray::V2rayService;

// RUST_LOG=INFO cargo run
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::init();

    //TODO: read command line arguments

    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let mut v2ray_proxy = http_proxy_service(
        &server.configuration,
        V2rayService::new(
            "one.one.one.one".to_string(),
            "1.1.1.1:443".to_string(),
            "/xyz".to_string(),
            false,
        ),
    );

    let cert_path = format!("{}/keys/one.one.one.one.pem", env!("CARGO_MANIFEST_DIR"));
    let key_path = format!(
        "{}/keys/one.one.one.one-key.pem",
        env!("CARGO_MANIFEST_DIR")
    );
    v2ray_proxy
        .add_tls("0.0.0.0:8999", &cert_path, &key_path)
        .unwrap();

    server.add_service(v2ray_proxy);
    server.run_forever();
}
