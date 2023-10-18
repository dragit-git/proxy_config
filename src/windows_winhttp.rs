use color_eyre::eyre::eyre;
use std::ffi::OsString;
use std::io::BufRead;
use std::os::windows::ffi::OsStringExt;
use log::error;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::GlobalFree;
use winapi::um::winhttp::*;
use crate::{Proxies, Proxy, ProxyConfig, ProxyConfigProvider, Scope};

impl ProxyConfigProvider for ProxyConfig {
    fn try_get() -> color_eyre::Result<Self> {
        let (proxies, scopes, _) = get_proxy_config()?;
        let proxies = proxies.ok_or(eyre!("No proxies found."))?;
        let scopes = scopes.ok_or(eyre!("No scopes found."))?;
        let proxies = Proxies::from_str(proxies)?;
        let scopes = Scope::from_str(scopes, proxies.clone())?;
        Ok(Self {
            proxies,
            scopes: vec![scopes],
        })
    }
}

impl Proxies {
    pub fn from_str(value: impl AsRef<str>) -> color_eyre::Result<Self> {
        let mut items = value.as_ref().split(":");
        let host = items.nth(0).ok_or(eyre!("No host found."))?;
        let port = items.nth(0).ok_or(eyre!("No port found."))?.parse::<u16>()?;
        let proxy = Proxy {
            host: host.to_string(),
            port,
            auth: None,
            enabled: true,
        };
        Ok(Proxies {
            http_proxy: Some(proxy.clone()),
            https_proxy: Some(proxy.clone()),
            socks_proxy: Some(proxy.clone()),
            exclude_simple_host_names: true,
        })
    }
}

impl Scope {
    pub fn from_str(value: impl AsRef<str>, proxies: Proxies) -> color_eyre::Result<Self> {
        let exceptions = value.as_ref().split(";").map(|s| s.to_string()).collect::<Vec<_>>();
        Ok(Self {
            interface: "".to_string(),
            proxies,
            exceptions,
        })
    }
}

fn get_proxy_config() -> color_eyre::Result<(Option<String>, Option<String>, Option<String>)> {
    unsafe {
        let mut proxy = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG::default();
        let result = WinHttpGetIEProxyConfigForCurrentUser(&mut proxy);

        if result == 0 {
            let e = GetLastError() as u32;
            error!("Error getting IE Proxy Config for Current User.");
            match e {
                1 => Err(eyre!("No Internet Explorer proxy settings can be found.")),

                12004 => Err(eyre!("An internal error has occurred.")),
                8 => Err(eyre!(
                    "Not enough memory was available to complete the requested operation."
                )),
                _ => Err(eyre!("Unknown error: {}.", e)),
            }
        } else {
            // release allocated string
            let mut proxies = None;
            if !proxy.lpszProxy.is_null() {
                proxies = Some(format!("{}", lpwstr_to_string(proxy.lpszProxy).unwrap_or_default()));
                GlobalFree(proxy.lpszProxy as _);
            }

            let mut scopes = None;
            if !proxy.lpszProxyBypass.is_null() {
                scopes = Some(format!(
                    "{}",
                    lpwstr_to_string(proxy.lpszProxyBypass).unwrap_or_default()
                ));
                GlobalFree(proxy.lpszProxyBypass as _);
            }
            let mut auto_url = None;
            if !proxy.lpszAutoConfigUrl.is_null() {
                auto_url = Some(format!(
                    "{}",
                    lpwstr_to_string(proxy.lpszAutoConfigUrl).unwrap_or_default()
                ));
                GlobalFree(proxy.lpszAutoConfigUrl as _);
            }
            Ok((proxies, scopes, auto_url))
        }
    }
}

fn lpwstr_to_string(lpwstr: *mut u16) -> Option<String> {
    if lpwstr.is_null() {
        return None;
    }

    let mut length = 0;
    unsafe {
        while *lpwstr.add(length) != 0 {
            length += 1;
        }
    }

    let slice = unsafe { std::slice::from_raw_parts(lpwstr, length) };

    let os_string = OsString::from_wide(slice);

    os_string.into_string().ok()
}

#[cfg(test)]
mod tests {
    use crate::windows_winhttp::get_proxy_config;

    #[test]
    fn test_get_config() {
        get_proxy_config().unwrap();
    }
}
