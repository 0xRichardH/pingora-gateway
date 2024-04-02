use crate::{
    prelude::*,
    services::{DefaultResponseFilter, FilterRequest, V2rayRequestFilter},
};
use std::{fs::File, io::Read, sync::Arc};

use log::debug;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub proxy_service: ProxyService,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProxyService {
    pub host_configs: Vec<HostConfig>,
    pub listen_addr: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct HostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_hostname: String,
    pub cert_path: String,
    pub key_path: String,
    filters: Vec<Filter>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Filter {
    DefaultResponseFilter,
    V2rayRequestFilter,
}

impl TryFrom<&str> for Config {
    type Error = toml::de::Error;

    fn try_from(contents: &str) -> Result<Self, Self::Error> {
        toml::from_str(contents)
    }
}

impl HostConfig {
    pub fn get_filters(&self) -> Vec<Arc<dyn FilterRequest>> {
        self.filters
            .iter()
            .map(|filter| filter.get_filter_fn())
            .collect()
    }
}

impl Filter {
    pub fn get_filter_fn(&self) -> Arc<dyn FilterRequest> {
        match self {
            Filter::DefaultResponseFilter => Arc::new(DefaultResponseFilter {}),
            Filter::V2rayRequestFilter => Arc::new(V2rayRequestFilter::new("/ws".to_string())),
        }
    }
}

pub fn load_config(path: &str) -> Result<Config> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = Config::try_from(contents.as_str())?;

    debug!("Config: {:?}", config.clone());

    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_load_config() -> Result<()> {
        // prepare test data
        let tmp_dir = tempdir::TempDir::new("test_load_config")?;
        let config_path = tmp_dir.path().join("config.toml");
        let mut f = File::create(&config_path)?;
        f.write_all(create_test_config_content().as_bytes())?;
        f.sync_all()?;

        // test
        let config = load_config(config_path.display().to_string().as_str())?;
        assert_eq!(
            config.proxy_service.listen_addr,
            String::from("0.0.0.0:443")
        );
        assert_eq!(config.proxy_service.host_configs.len(), 2);

        let host_config_1 = config.proxy_service.host_configs[0].clone();
        assert_eq!(host_config_1.proxy_addr, String::from("1.1.1.1:443"));
        assert!(host_config_1.proxy_tls);
        assert_eq!(
            host_config_1.proxy_hostname,
            String::from("one.one.one.one")
        );
        assert_eq!(host_config_1.cert_path, String::from("one.one.one.one.pem"));
        assert_eq!(
            host_config_1.key_path,
            String::from("one.one.one.one-key.pem")
        );
        assert_eq!(host_config_1.get_filters().len(), 2);
        assert_eq!(host_config_1.filters[0], Filter::DefaultResponseFilter);
        assert_eq!(host_config_1.filters[1], Filter::V2rayRequestFilter);

        let host_config_2 = config.proxy_service.host_configs[1].clone();
        assert_eq!(host_config_2.proxy_addr, String::from("1.1.1.2:443"));
        assert!(!host_config_2.proxy_tls);
        assert_eq!(
            host_config_2.proxy_hostname,
            String::from("one.one.one.two")
        );
        assert_eq!(host_config_2.cert_path, String::from("one.one.one.two.pem"));
        assert_eq!(
            host_config_2.key_path,
            String::from("one.one.one.two-key.pem")
        );
        assert_eq!(host_config_2.get_filters().len(), 1);
        assert_eq!(host_config_2.filters[0], Filter::DefaultResponseFilter);

        // delete temp dir
        tmp_dir.close()?;
        Ok(())
    }

    fn create_test_config_content() -> String {
        String::from(
            r#"
            [proxy_service]
            listen_addr = "0.0.0.0:443"

            [[proxy_service.host_configs]]
            proxy_addr = "1.1.1.1:443"
            proxy_tls = true
            proxy_hostname = "one.one.one.one"
            cert_path = "one.one.one.one.pem"
            key_path = "one.one.one.one-key.pem"
            filters = ["DefaultResponseFilter", "V2rayRequestFilter"]

            [[proxy_service.host_configs]]
            proxy_addr = "1.1.1.2:443"
            proxy_tls = false
            proxy_hostname = "one.one.one.two"
            cert_path = "one.one.one.two.pem"
            key_path = "one.one.one.two-key.pem"
            filters = ["DefaultResponseFilter"]
        "#,
        )
    }
}
