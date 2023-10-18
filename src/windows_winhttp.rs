use color_eyre::eyre::eyre;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::GlobalFree;
use winapi::um::winhttp::*;

fn get_proxy_config() -> color_eyre::Result<()> {
    unsafe {
        // 获取代理配置
        let mut proxy = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG::default();
        let result = WinHttpGetIEProxyConfigForCurrentUser(&mut proxy);

        if result == 0 {
            let e = GetLastError() as u32;
            println!("Error getting IE Proxy Config for Current User.");
            match e {
                1 => Err(eyre!("No Internet Explorer proxy settings can be found.")),

                12004 => Err(eyre!("An internal error has occurred.")),
                8 => Err(eyre!(
                    "Not enough memory was available to complete the requested operation."
                )),
                _ => Err(eyre!("Unknown error: {}.", e)),
            }
        } else {
            println!("{}", proxy.fAutoDetect);
            // 释放动态分配的字符串
            if !proxy.lpszProxy.is_null() {
                println!("{}", lpwstr_to_string(proxy.lpszProxy).unwrap_or_default());
                GlobalFree(proxy.lpszProxy as _);
            }
            if !proxy.lpszProxyBypass.is_null() {
                println!(
                    "{}",
                    lpwstr_to_string(proxy.lpszProxyBypass).unwrap_or_default()
                );
                GlobalFree(proxy.lpszProxyBypass as _);
            }
            if !proxy.lpszAutoConfigUrl.is_null() {
                println!(
                    "{}",
                    lpwstr_to_string(proxy.lpszAutoConfigUrl).unwrap_or_default()
                );
                GlobalFree(proxy.lpszAutoConfigUrl as _);
            }
            Ok(())
        }
    }
}

fn lpwstr_to_string(lpwstr: *mut u16) -> Option<String> {
    if lpwstr.is_null() {
        return None;
    }

    // 确定字符串的长度
    let mut length = 0;
    unsafe {
        while *lpwstr.add(length) != 0 {
            length += 1;
        }
    }

    // 构建一个 slice（切片）
    let slice = unsafe { std::slice::from_raw_parts(lpwstr, length) };

    // 转换为 OsString
    let os_string = OsString::from_wide(slice);

    // 转换为 String
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
