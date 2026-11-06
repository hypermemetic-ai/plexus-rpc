# plexus-rpc

**One dependency for a Plexus RPC service.**

```toml
[dependencies]
plexus-rpc = "0.8"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }   # for your `main`
```

`plexus-rpc` brings the Plexus crates at versions that work together, its
version in lockstep with plexus-core's: when plexus-core moves to 0.9, so does
this crate, and an app bumps one number.

| module | crate | what's in it |
|---|---|---|
| `plexus_rpc::core` | plexus-core (feature `server`, on by default) | `DynamicHub`, `Activation`, dispatch, schemas, the in-process transport |
| `plexus_rpc::macros` | plexus-macros | `#[activation]`, `#[method]`, `#[child]`, `#[derive(PlexusRequest, Credentials, HandleEnum)]` — the attribute macros also at the root (`plexus_rpc::activation`) |
| `plexus_rpc::auth_core` | plexus-auth-core | `AuthContext`, `Principal`, `Credential<T>`, grants, forwarding, audit |
| `plexus_rpc::client` | plexus-client | the runtime of the client `#[activation]` writes for you |
| `plexus_rpc::transport` | plexus-transport (feature `transport`, on by default) | `TransportServer`: WebSocket, HTTP, stdio |
| `plexus_rpc::prelude` | — | what an activation and its `main` usually name |

The macros' expansions name `async_trait`, `futures`, `jsonrpsee`, `schemars`,
`serde`, `serde_json`, `tokio` and `uuid`. With `plexus-rpc` a dependency,
plexus-macros (0.8.1 and later) routes every one of them through
`plexus_rpc::__deps`, so your manifest lists none of them: only what your own
code uses. jsonrpsee is an implementation detail behind this crate.

## A service

```rust
use std::sync::Arc;
use plexus_rpc::prelude::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Tick { pub n: u32 }

impl FromClientError for Tick {
    fn from_client_error(_: ClientError) -> Self { Tick { n: 0 } }
}

#[derive(Clone, Default)]
pub struct EchoActivation;

/// Say things back.
#[activation(namespace = "echo", version = "1.0.0")]
impl EchoActivation {
    /// Ticks from 1 up to `to`.
    #[method]
    pub async fn count(&self, /** the last tick */ to: u32) -> impl Stream<Item = Tick> + Send + 'static {
        futures::stream::iter((1..=to).map(|n| Tick { n }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = Arc::new(DynamicService::new("myservice").register(EchoActivation));
    TransportServer::builder(service, |s| DynamicService::arc_into_rpc_module(s).map_err(Into::into))
        .with_websocket(4444)
        .build()
        .await?
        .serve()
        .await
}
```

```bash
synapse myservice echo count --to 3
```

This one uses serde, schemars, futures and anyhow by name in its own code, so
it lists them. An app that would rather not names them through the umbrella:
`plexus_rpc::__deps::futures`, and on its own types
`#[serde(crate = "plexus_rpc::__deps::serde")]` and
`#[schemars(crate = "plexus_rpc::__deps::schemars")]` —
[`tests/umbrella-app`](tests/umbrella-app) is such an app, its manifest
`plexus-rpc` and `tokio` and nothing else.

`#[activation]` also writes the activation's trait (`Echo`) and a client
(`EchoClient<T: Transport>`) that implements it. In process:

```rust
let client = EchoClient::new(ServiceTransport::anonymous(service.clone()));
let ticks: Vec<Tick> = client.count(3).collect().await;
```

Over a WebSocket (feature `client-ws`):

```rust
let client = EchoClient::new(plexus_rpc::client::ws::WsTransport::connect("ws://127.0.0.1:4444", "myservice").await?);
```

`DynamicService` and `ServiceTransport` are plexus-core's `DynamicHub` and
`HubTransport`, by the names the ecosystem uses.

## Features

| feature | default | what it turns on |
|---|---|---|
| `server` | yes | `plexus_rpc::core` (plexus-core), the prelude's `DynamicService` and friends, `__deps::jsonrpsee`, `CAPABILITIES` |
| `transport` | yes | `plexus_rpc::transport` (plexus-transport); turns on `server` |
| `child-ws` | yes | plexus-core's remote children over a WebSocket; turns on `server` |
| `child-wss` | | … over TLS |
| `clap` | | plexus-core's clap integration |
| `client-ws` | | `plexus_rpc::client::ws::WsTransport`, a native WebSocket client |
| `mcp`, `mcp-gateway`, `http-gateway`, `sqlite-sessions` | | plexus-transport's features of those names |
| `auth-adapters` | | plexus-auth-core's `adapters`: OIDC over reqwest, nostr, agent keys |
| `auth-test-support` | | plexus-auth-core's `test-support`, for a crate's dev-dependencies |

A crate that only defines activations, or only calls them, can drop the
defaults: `plexus-rpc = { version = "0.8", default-features = false }` — plus
`features = ["child-ws"]` (or `"server"`) when it still needs plexus-core.

## In the browser (wasm32)

With `default-features = false` (0.8.1 and later) the umbrella is
plexus-macros, plexus-client, plexus-auth-core and `__deps`, and it builds
for `wasm32-unknown-unknown`. An API crate that is a service natively and a
client in the browser gives itself a `server` feature that turns on the
umbrella's; `#[activation]` gates its server half on the crate's own
`server` feature, so with it off the trait and the generated client are
what is left:

```toml
[features]
default = ["server"]
server = ["plexus-rpc/server"]

[dependencies]
plexus-rpc = { version = "0.8.1", default-features = false }
```

`tests/umbrella-wasm` is that crate, and `tests/wasm32_check.rs` builds both
for wasm32. getrandom (uuid's v4) needs
`--cfg getrandom_backend="wasm_js"` in the target's rustflags, e.g. in
`.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]
```

## The guard

`tests/umbrella-app` is an umbrella-only app: an activation with one,
many and fallible methods, `#[derive(PlexusRequest)]`, `#[derive(Credentials)]`,
`#[derive(HandleEnum)]`, a `main` that serves it, and tests that call it
through its trait, in process, over a WebSocket, and through the jsonrpsee
server trait the macro writes. Its manifest lists `plexus-rpc` and tokio
(renamed, so `::tokio` is not in its extern prelude either) and a test holds
it to that, so a macro path that escapes the umbrella fails its build.

```bash
cargo test --workspace
```

## Capability manifest

A service can embed `plexus_rpc::CAPABILITIES` in its `_info` so generic
clients negotiate features instead of guessing from version strings: the
bundled crates' versions and a list of named feature flags.

## License

MIT.
