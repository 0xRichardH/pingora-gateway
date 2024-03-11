use pingora::{proxy::http_proxy_service, server::Server};
use pingora_gateway::services::v2ray::V2rayService;

// RUST_LOG=INFO cargo run
fn main() {
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
    v2ray_proxy.add_tcp("0.0.0.0:8999");

    let mut v2ray_proxy_2 = http_proxy_service(
        &server.configuration,
        V2rayService::new(
            "one.one.one.two".to_string(),
            "1.1.1.1:443".to_string(),
            "/abc".to_string(),
            false,
        ),
    );
    v2ray_proxy_2.add_tcp("0.0.0.0:8998");

    server.add_service(v2ray_proxy);
    server.add_service(v2ray_proxy_2);
    server.run_forever();
}
