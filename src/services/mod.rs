mod proxy;
mod service;

use std::collections::HashMap;

pub use service::proxy_service_tls;

#[derive(Clone)]
pub struct HostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_hostname: String,
    pub cert_path: String,
    pub key_path: String,
}

type HostName = String;
pub type HostConfigs = HashMap<HostName, HostConfig>;
// TODO: impl From for HostConfigs
