# pingora-gateway

A configurable TLS-terminating reverse proxy built on Cloudflare's [Pingora](https://github.com/cloudflare/pingora).

It listens on a single address, terminates TLS per hostname (SNI), routes requests to upstream backends by `Host` header, and runs a pluggable chain of request filters per host before proxying.

## Features

- **SNI-based TLS termination.** One listener serves many hostnames, each with its own certificate.
- **Host-header routing.** Each request is matched to an upstream by its `Host` header.
- **Pluggable filter chain.** Per-host filters run before the upstream call. Any filter can short-circuit the request and write its own response.
- **Upstream TLS.** Toggle `proxy_tls` per host to proxy to HTTPS or plain HTTP backends.
- **Header rewriting.** Replaces the `Server` header with `Cloudflare` and strips `alt-svc` (Pingora does not speak HTTP/3 to clients).

## Architecture

A request flows through five stages:

1. **TLS handshake** (`services/service.rs`). A `Callback` preloads each host's cert and key at startup. On each handshake, it reads the SNI, finds the matching entry, and sets that cert and key on the connection. One listener thus serves many hostnames.
2. **Request filter** (`services/proxy.rs::ProxyService::request_filter`). Extracts the `Host` header, looks up the matching `HostConfig`, stashes it and the request path in `ProxyCtx`, then runs each filter in order. A filter returning `Ok(true)` writes its own response and stops the pipeline. Unknown hosts get `400 Bad Request`.
3. **Upstream peer** (`ProxyService::upstream_peer`). Builds an `HttpPeer` from `ctx.host_config` — the upstream address, TLS flag, and SNI/Host to send upstream.
4. **Response filter** (`ProxyService::response_filter`). Rewrites the upstream response: sets `Server: Cloudflare`, removes `alt-svc`.
5. **Logging** (`ProxyService::logging`). Logs the response code and a request summary.

```
client
  │  TLS (SNI selects cert)
  ▼
request_filter  ── Host header → HostConfig lookup → filter chain
  │
  ▼
upstream_peer  ── proxy_addr, proxy_tls, proxy_hostname
  │
  ▼
upstream
  │
  ▼
response_filter ── rewrite Server, strip alt-svc
  │
  ▼
client
```

### Source layout

```
src/
  main.rs              entry point: load config, build HostConfigs, register the proxy service
  config.rs            TOML schema + Filter enum (serde tag = "type", content = "args")
  prelude.rs           Result alias + W<T> wrapper
  services/
    mod.rs             HostConfig / HostConfigs (HashMap<hostname, HostConfig>)
    service.rs         TLS callback + proxy_service_tls wiring
    proxy.rs           ProxyService implementing pingora::ProxyHttp
    request_filter.rs  FilterRequest trait + built-in filters
```

## Configuration

Configure the gateway with a TOML file (default path: `config.toml` next to the binary, or set `CARGO_MANIFEST_DIR`).

```toml
[proxy_service]
listen_addr = "0.0.0.0:8999"
# Optional: trusted root CA for verifying upstream TLS
root_cert_path = "root.pem"

# one.one.one.one
[[proxy_service.host_configs]]
proxy_addr     = "1.1.1.1:443"   # upstream address
proxy_tls      = true             # TLS to upstream
proxy_hostname = "one.one.one.one"  # SNI/Host sent upstream; also the routing key
cert_path      = "./keys/one.one.one.one.pem"
key_path       = "./keys/one.one.one.one-key.pem"

[[proxy_service.host_configs.filters]]
type = "DefaultResponseFilter"

[[proxy_service.host_configs.filters]]
type = "SimplePathFilter"
args = "/ws"

# one.one.one.two
[[proxy_service.host_configs]]
proxy_addr     = "1.1.1.2:443"
proxy_tls      = false
proxy_hostname = "one.one.one.two"
cert_path      = "./keys/one.one.one.two.pem"
key_path       = "./keys/one.one.one.two-key.pem"

[[proxy_service.host_configs.filters]]
type = "DefaultResponseFilter"
```

### Fields

| Field | Description |
| --- | --- |
| `proxy_service.listen_addr` | Address the gateway listens on. |
| `proxy_service.root_cert_path` | Optional path to a trusted root CA bundle for verifying upstream TLS. |
| `host_configs[].proxy_addr` | Upstream `host:port`. |
| `host_configs[].proxy_tls` | Use TLS when connecting to the upstream. |
| `host_configs[].proxy_hostname` | Hostname sent upstream (SNI/Host); also the key for Host-header routing. |
| `host_configs[].cert_path` | PEM certificate for TLS termination. |
| `host_configs[].key_path` | PEM private key for TLS termination. |
| `host_configs[].filters` | Ordered list of filters (see below). |

### Built-in filters

Filters implement the `FilterRequest` trait. Each filter runs in order; returning `Ok(true)` writes the response and stops the chain.

| Filter | Args | Behavior |
| --- | --- | --- |
| `DefaultResponseFilter` | none | If the path is `/`, respond `200` with body `Connecting...` and stop. Otherwise pass through. |
| `SimplePathFilter` | path string | Allow only requests whose path starts with `path`; otherwise respond `404` and stop. |

### Adding a filter

1. Implement `FilterRequest` in `src/services/request_filter.rs`.
2. Add a variant to the `Filter` enum in `src/config.rs` with `#[serde(tag = "type", content = "args")]`.
3. Map it to an `Arc<dyn FilterRequest>` in `Filter::get_filter_fn`.

## Run

### Docker

```shell
docker compose -f ./docker-compose-example.yaml up
```

The example compose file mounts `config.example.toml` as `config.toml` and a `./keys/` directory for certificates, and maps host port `8080` to the gateway's `8999`.

### Native

```shell
cargo build --release
RUST_LOG=debug ./target/release/pingora-gateway
```

The binary loads `config.toml` from `CARGO_MANIFEST_DIR`; override by setting that env var (the Docker image sets it to `/gateway`).

## Example

With the example config proxying `one.one.one.one` to Cloudflare's DNS endpoint:

```shell
curl --connect-to one.one.one.one:443:127.0.0.1:8080 https://one.one.one.one/ -vk
```

## License

MIT