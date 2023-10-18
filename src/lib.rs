#[cfg(target_os = "macos")]
mod macos_cfnetwork;

#[cfg(target_os = "windows")]
mod windows_winhttp;

#[cfg(target_os = "linux")]
mod linux_desktop;

#[cfg(target_os = "macos")]
pub use macos_cfnetwork::*;

#[cfg(target_os = "windows")]
pub use windows_winhttp::*;

#[cfg(target_os = "linux")]
pub use linux_desktop::*;

#[derive(Debug, Clone)]
pub struct Auth {
    pub user: String,
    pub password: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Proxy {
    pub host: String,
    pub port: u16,
    pub auth: Option<Auth>,
    pub enabled: bool,
}

#[derive(Debug, Default, Clone)]
pub struct Proxies {
    pub http_proxy: Option<Proxy>,
    pub https_proxy: Option<Proxy>,
    pub socks_proxy: Option<Proxy>,
    pub exclude_simple_host_names: bool,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub interface: String,
    pub proxies: Proxies,
    pub exceptions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub proxies: Proxies,
    pub scopes: Vec<Scope>,
}

pub trait ProxyConfigProvider: Clone {
    fn try_get() -> color_eyre::Result<Self>;
}

#[cfg(test)]
mod test {
    use crate::{ProxyConfig, ProxyConfigProvider};

    #[test]
    fn test_proxy_config() -> color_eyre::Result<()> {
        color_eyre::install()?;
        let proxy_config = ProxyConfig::try_get().unwrap();
        println!("{:#?}", proxy_config);
        Ok(())
    }
}
