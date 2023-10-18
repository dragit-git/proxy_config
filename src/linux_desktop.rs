use crate::{Auth, Proxies, Proxy, ProxyConfig, ProxyConfigProvider};
use color_eyre::eyre::eyre;
use std::env;
use std::process::Command;
use std::str::FromStr;

impl ProxyConfigProvider for ProxyConfig {
    fn try_get() -> color_eyre::Result<Self> {
        let desktop = env::var("XDG_CURRENT_DESKTOP")?;
        match desktop {
            desktop if desktop.to_lowercase().contains("gnome") => {
                let settings = get_gnome_settings()?;
                let settings = settings.lines().collect::<Vec<_>>();
                Ok(ProxyConfig {
                    proxies: Proxies::from_gnome_settings(&settings),
                    scopes: vec![],
                })
            }
            _ => Err(eyre!("Unsupported desktop environment")),
        }
    }
}

pub trait FromGnomeSettings: Clone {
    type This;
    fn from_gnome_settings(settings: &[&str]) -> Self::This;
}

impl FromGnomeSettings for Proxies {
    type This = Self;

    fn from_gnome_settings(settings: &[&str]) -> Self::This {
        Self {
            http_proxy: Proxy::from_gnome_settings("http", settings),
            https_proxy: Proxy::from_gnome_settings("https", settings),
            socks_proxy: Proxy::from_gnome_settings("socks", settings),
            exclude_simple_host_names: true,
        }
    }
}

impl Proxy {
    fn from_gnome_settings(prefix: &str, settings: &[&str]) -> Option<Proxy> {
        let mut host = None;
        let mut port = None;
        let mut user = None;
        let mut password = None;
        let mut use_auth = false;
        settings.iter().for_each(|line| {
            let items = line.split(" ").collect::<Vec<&str>>();
            let (path, key, value) = (items[0], items[1], items[2]);
            if path.ends_with(prefix) && key == "host" {
                host = Some(value.replace("'", ""));
            }
            if path.ends_with(prefix) && key == "port" {
                port = Some(u16::from_str(value).unwrap());
            }
            if key == "use-authentication" && value == "true" {
                use_auth = true;
            }
            if use_auth && key.ends_with("user") {
                user = Some(value.to_string());
            }
            if use_auth && key.ends_with("password") {
                password = Some(value.to_string());
            }
        });
        let mut auth = None;
        if use_auth {
            if let (Some(user), password) = (user, password) {
                auth = Some(Auth { user, password })
            }
        }
        if let (Some(host), Some(port)) = (host, port) {
            Some(Proxy {
                host,
                port,
                auth,
                enabled: true,
            })
        } else {
            None
        }
    }
}

fn get_gnome_settings() -> color_eyre::Result<String> {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .args(["gsettings list-recursively org.gnome.system.proxy"]);
    let output = command.output()?;
    match output.status.success() {
        false => Err(eyre!(format!(
            "{:?} failed: {}",
            command,
            String::from_utf8_lossy(&output.stderr)
        ))),
        true => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gnome_settings() -> color_eyre::Result<()> {
        color_eyre::install()?;
        let config = ProxyConfig::try_init()?;
        println!("{:#?}", config);
        Ok(())
    }
}
