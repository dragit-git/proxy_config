use std::ffi::{c_char, c_void, CStr};

use crate::{Auth, Proxies, Proxy, ProxyConfig, ProxyConfigProvider, Scope};
use core_foundation::dictionary::CFDictionaryRef;
#[allow(unused)]
use fruity::cf_network;
use objc::runtime::*;
use objc::*;
use serde_json::Value;

extern "C" {
    fn CFNetworkCopySystemProxySettings() -> CFDictionaryRef;
}

fn get_proxy_config() -> color_eyre::Result<String> {
    unsafe {
        // get proxy config
        let proxy_config = CFNetworkCopySystemProxySettings();

        let ns_json: &Class = class!(NSJSONSerialization);
        let ns_string: &Class = class!(NSString);
        let mut error: *mut c_void = std::ptr::null_mut();

        // serialize proxy config to json
        let data: *mut Object =
            msg_send![ns_json, dataWithJSONObject:proxy_config options:1 error:&mut error];

        // convert json data to string
        let string: *mut Object = msg_send![ns_string, alloc];
        let utf8_string: *mut Object = msg_send![string, initWithData:data encoding:4]; // 4 is NSUTF8StringEncoding
        let c_string: *const c_char = msg_send![utf8_string, cString];
        Ok(CStr::from_ptr(c_string).to_str()?.to_string())
    }
}

impl ProxyConfigProvider for ProxyConfig {
    fn try_get() -> color_eyre::Result<Self> {
        let proxy_config_str = get_proxy_config()?;
        let proxy_config_value: Value = serde_json::from_str(proxy_config_str.as_str()).unwrap();
        let proxy_config = Self {
            proxies: Proxies::from_value(&proxy_config_value),
            scopes: proxy_config_value["__SCOPED__"]
                .as_object()
                .map(|scoped| {
                    scoped
                        .iter()
                        .map(|(key, value)| Scope::from_value(key, value))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        };
        Ok(proxy_config)
    }
}

impl Proxies {
    pub fn from_value(value: &Value) -> Self {
        Self {
            http_proxy: Proxy::from_value("HTTP", value),
            https_proxy: Proxy::from_value("HTTPS", value),
            socks_proxy: Proxy::from_value("SOCKS", value),
            exclude_simple_host_names: value["ExcludeSimpleHostnames"].as_u64() == Some(1),
        }
    }
}

impl Scope {
    pub fn from_value(interface: &str, value: &Value) -> Self {
        Self {
            interface: interface.to_string(),
            proxies: Proxies::from_value(value),
            exceptions: {
                let mut exceptions = vec![];
                if let Some(value) = value.as_object() {
                    value["ExceptionsList"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .for_each(|value| exceptions.push(value.as_str().unwrap().to_string()));
                }
                exceptions
            },
        }
    }
}

impl Proxy {
    pub fn from_value(prefix: &str, value: &Value) -> Option<Self> {
        match value.as_object() {
            None => None,
            Some(value) => {
                let mut host = None;
                let mut port = None;
                let mut user = None;
                let mut password = None;
                let mut enabled = None;
                for (key, value) in value {
                    if key.ends_with("Proxy") && &key[..(key.len() - 5)] == prefix {
                        host = value.as_str().map(|v| v.to_string());
                    }
                    if key.ends_with("Port") && &key[..(key.len() - 4)] == prefix {
                        port = value.as_u64().map(|v| v as u16);
                    }
                    if key.ends_with("User") && &key[..(key.len() - 4)] == prefix {
                        user = value.as_str().map(|v| v.to_string());
                    }
                    if key.ends_with("Password") && &key[..(key.len() - 8)] == prefix {
                        password = value.as_str().map(|v| v.to_string());
                    }
                    if key.ends_with("Enable") && &key[..(key.len() - 6)] == prefix {
                        enabled = value.as_u64().map(|v| v == 1);
                    }
                }
                let auth = match (user, password) {
                    (Some(user), password) => Some(Auth { user, password }),
                    _ => None,
                };
                if let (Some(host), Some(port), Some(enabled)) = (host, port, enabled) {
                    Some(Self {
                        host,
                        port,
                        auth,
                        enabled,
                    })
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod test {
    use crate::{ProxyConfig, ProxyConfigProvider};

    #[test]
    fn check_proxy() -> color_eyre::Result<()> {
        color_eyre::install()?;
        let proxy_config = ProxyConfig::try_get()?;
        println!("{:#?}", proxy_config);
        Ok(())
    }
}
