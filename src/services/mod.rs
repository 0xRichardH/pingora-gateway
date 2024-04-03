mod proxy;
mod request_filter;
mod service;

pub use request_filter::FilterRequest;
pub use request_filter::{DefaultResponseFilter, SimplePathFilter};
pub use service::proxy_service_tls;
use std::collections::HashMap;

use std::sync::Arc;

use crate::prelude::*;

#[derive(Clone)]
pub struct HostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_hostname: String,
    pub cert_path: String,
    pub key_path: String,
    pub filters: Vec<Arc<dyn FilterRequest>>,
}

type HostName = String;
pub type HostConfigs = HashMap<HostName, HostConfig>;
impl From<Vec<(HostName, HostConfig)>> for W<HostConfigs> {
    fn from(array: Vec<(HostName, HostConfig)>) -> Self {
        let configs = array
            .into_iter()
            .fold(HostConfigs::new(), |mut acc, (name, config)| {
                acc.insert(name, config);
                acc
            });
        W(configs)
    }
}
