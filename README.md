# Proxy Config

This is a simple library to help you get the proxy configuration from the system.

# Why

Usually you can get the proxy configuration through environment variables

Such as 

* `http_proxy`
* `https_proxy`
* `all_proxy`
* `no_proxy`
* etc.

But in a GUI environment?

The environment variables are not always inherited to the shell

And when you use some proxy software with gui, such as `clash`/`surge`

They will not set the proxy to environment variables for system's desktop,
but directly sets the proxy configuration for the system's desktop manager

# Usage

```toml
[dependencies]
proxy_config = "0.1"
```

```rust
use proxy_config::ProxyConfig;

let proxy_config = ProxyConfig::try_get().unwrap();
```

# Supported Platforms

* macOS
* Linux with GNOME
* Windows (in progress)
