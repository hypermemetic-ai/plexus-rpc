# plexus-rpc

**Two dependencies. Schema-driven RPC for distributed systems in Rust + Haskell.**

`plexus-rpc` is the umbrella crate for the Plexus RPC framework. Add two lines to your `Cargo.toml` and you get the verified-compatible set: dispatch core, procedural macros, sealed identity / credential / tenant / audit primitives, and the WebSocket / HTTP / stdio server runtime.

```toml
[dependencies]
plexus-rpc = "0.2"
jsonrpsee = { version = "0.26", features = ["server", "macros"] }
```

Why two and not one? `plexus-macros` resolves every dependency its expansions
emit (`uuid`, `serde`, `serde_json`, `schemars`, `futures`, `tokio`,
`async-trait`) through `plexus_rpc::__deps` re-exports, so none of those need
to be in your manifest. The single exception is `jsonrpsee`: plexus-macros
delegates server-trait generation to jsonrpsee's own `#[rpc]` proc-macro,
which looks `jsonrpsee` up in *your* Cargo.toml and hard-errors if it isn't
there. Inlining that expansion (true one-dependency) is a tracked follow-up.
The version must stay semver-compatible with the umbrella's pin (`0.26`).

Define methods in plain Rust. The macro extracts JSON Schema from your signatures and rustdoc, registers handlers, and lets `synapse` discover the whole surface at runtime. No route tables, no schema files, no codegen step.

## Hello, plexus

A working server in one file. Run it; talk to it from the [synapse](https://github.com/hypermemetic/plexus-synapse) CLI.

```rust
// examples/echo.rs
use async_stream::stream;
use futures::Stream;
use plexus_core::plexus::DynamicHub;
use plexus_rpc::transport::TransportServer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EchoEvent {
    Echo { message: String, count: u32 },
}

pub struct Echo;

#[plexus_rpc::macros::activation(
    namespace = "echo",
    version = "1.0.0",
    description = "Echo messages back"
)]
impl Echo {
    /// Echo a message back the specified number of times.
    #[plexus_rpc::macros::method]
    async fn echo(
        &self,
        /// The message to echo
        message: String,
        /// Number of times to repeat
        count: u32,
    ) -> impl Stream<Item = EchoEvent> + Send + 'static {
        stream! {
            for i in 0..count {
                yield EchoEvent::Echo { message: message.clone(), count: i + 1 };
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hub = Arc::new(DynamicHub::new("echo").register(Echo));

    let rpc_converter = |arc| {
        DynamicHub::arc_into_rpc_module(arc)
            .map_err(|e| anyhow::anyhow!("rpc module: {e}"))
    };

    println!("listening on ws://127.0.0.1:4444");
    TransportServer::builder(hub, rpc_converter)
        .with_websocket(4444)
        .build().await?
        .serve().await
}
```

Note: the example above (like the shipped `examples/echo.rs`) uses the
direct-deps layout — `plexus_core`, `futures`, `serde`, `schemars`,
`async_stream`, `tokio` imported at the crate root — which still works
unchanged. In the two-dependency umbrella layout, the same surface resolves
through the re-exports instead:

- `plexus_rpc::core::plexus::DynamicHub`, `plexus_rpc::macros::activation` —
  macro-emitted code detects the umbrella and resolves `plexus-core` /
  third-party paths through it automatically.
- Your own event enums: derive through the re-exports and point the derive
  support code at them —
  `#[derive(plexus_rpc::__deps::serde::Serialize, plexus_rpc::__deps::schemars::JsonSchema)]`
  plus `#[serde(crate = "plexus_rpc::__deps::serde")]` and
  `#[schemars(crate = "plexus_rpc::__deps::schemars")]`.
- `#[tokio::main]` is the one macro that cannot resolve through a re-export
  (it emits bare `tokio` paths); use
  `plexus_rpc::__deps::tokio::runtime::Builder` for `main`, or add `tokio`
  as a direct dependency.

```bash
$ cargo run --example echo
listening on ws://127.0.0.1:4444
```

In another terminal:

```bash
$ synapse echo echo --message "hello" --count 3
message: hello
count: 1

message: hello
count: 2

message: hello
count: 3
```

The default `synapse` registry sits at `localhost:4444` — start your backend on that port and the CLI discovers it automatically.

## Installing synapse

Synapse is the CLI client that derives commands, help, and validation from your backend's schema. Install it from Hackage:

```bash
cabal update
cabal install plexus-synapse
```

Or build from source:

```bash
git clone https://github.com/hypermemetic/plexus-synapse
cd plexus-synapse && cabal install
```

With the binary on your `PATH`:

```bash
$ synapse                # lists registered backends at localhost:4444
$ synapse echo           # lists the methods on the echo activation
$ synapse --schema echo  # raw JSON Schema for the activation
$ synapse --emit-ir echo > echo.ir.json   # IR for codegen
```

## What's in the box

The umbrella re-exports four crates as namespaced modules:

| Re-export | Source crate | Contents |
|---|---|---|
| `plexus_rpc::auth_core` | `plexus-auth-core` | `AuthContext`, `Principal`, sealed `Credential<T>`, `Tenant` + `TenantResolver`, `ForwardPolicy`, `AuditRecord` + `AuditSink`, `BackendAuthCapabilities` |
| `plexus_rpc::core` | `plexus-core` | `DynamicHub`, `Activation` trait, `MethodSchema`, credential wire envelope, `ChildRouter`, hub builders (`with_auth_capabilities`, `with_forward_policy`) |
| `plexus_rpc::macros` | `plexus-macros` | `#[activation]`, `#[method]`, `#[child]`, `#[derive(Credentials)]`, `#[from_auth]` |
| `plexus_rpc::transport` | `plexus-transport` (default-on feature `transport`) | `TransportServer`, WebSocket / HTTP / stdio server runtime |

Drop the `transport` feature when you're building ahead-of-time codegen / embedded / WASM consumers that only need the type and dispatch surface:

```toml
plexus-rpc = { version = "0.2", default-features = false }
```

## The doc-comment-first method shape

Doc comments on the function and each parameter feed the JSON Schema description automatically:

```rust
#[plexus_rpc::macros::activation(namespace = "calc", version = "1.0.0")]
impl Calc {
    /// Add two integers.
    #[plexus_rpc::macros::method]
    async fn add(
        &self,
        /// Left-hand operand
        a: i64,
        /// Right-hand operand
        b: i64,
    ) -> impl Stream<Item = CalcEvent> + Send + 'static {
        stream! { yield CalcEvent::Result { value: a + b } }
    }
}
```

If you need to override (e.g. to differ from the rustdoc you want shown to library users), the explicit attribute syntax still works and wins when set:

```rust
#[plexus_rpc::macros::method(
    description = "Wire description, separate from the rustdoc",
    params(a = "Override of the param doc")
)]
```

## Auth, credentials, tenancy (one-line summary each)

The umbrella ships the AUTHZ wave-2 primitives. Each is opt-in — your existing backends keep working unchanged.

- **Auth capability advertisement** — `DynamicHub::with_auth_capabilities(...)` makes `_info` describe your supported mechanisms (Bearer / Cookie / OIDC / Anonymous) so generic clients can discover them.
- **Sealed identity** — `AuthContext`, `Principal`, `VerifiedUser` are sealed at the crate boundary; activation code receives references, never constructs them.
- **Credentials as return values** — a method can return `Credential<T>` from any response struct. The framework intercepts at the serialization boundary, replaces the value with a `{"$credential": "<id>"}` sentinel, routes the real value through a sidecar, and (with `attach_as = "cookie:<name>"`, e.g. `attach_as = "header:authorization"`) projects it to a `Set-Cookie` / response header. Generated TypeScript/Rust clients auto-store and auto-attach.
- **Tenant isolation** — `Tenant` is a sealed identity primitive; `ClaimTenantResolver` derives it from a JWT claim; `Tenanted<S>` + `Scoped<'a, S>` wrap your storage in a tenant boundary the type system enforces.
- **Forwarding policy** — `DynamicHub::with_forward_policy(namespace, policy)` declares how the caller's identity propagates on cross-boundary calls (`IdentityOnly`, `PassThrough`, `Anonymous`, or your own).
- **Audit** — `AuditRecord` + `AuditSink` trait + `TracingAuditSink` default sink emits to the `plexus::audit` tracing target.

## Capability manifest

Backends embed `plexus_rpc::CAPABILITIES` in their `_info` response so generic clients negotiate features instead of guessing from version strings.

```rust
let info = serde_json::json!({
    "backend": "my-backend",
    "capabilities": plexus_rpc::CAPABILITIES,
});
```

`CAPABILITIES.features` is a stable list of named flags (`"credentials"`, `"forward_policy"`, `"tenant"`, etc.). Tooling branches on flags, not versions.

## Code generation

`synapse --emit-ir <backend>` produces a structured IR JSON. `hub-codegen` consumes it to produce typed TypeScript and Rust clients with:

- Method signatures derived 1:1 from the backend
- Auto-storage of returned credentials on a `SessionRegistry`
- Auto-attach on methods declaring `requires_credential`
- Streaming responses surfaced as `AsyncIterable` (TS) / `impl Stream` (Rust)
- `///` doc comments threaded into the generated docstrings

See [`synapse-cc`](https://github.com/hypermemetic/synapse-cc) for the orchestrator that wires synapse + hub-codegen into one `cargo`-style build flow.

## Examples

The `examples/` directory has a runnable server:

- `examples/echo.rs` — the minimal hello-world above

It binds to `127.0.0.1:4444` by default so the bare `synapse` invocation finds it.

```bash
cargo run --example echo
```

## What plexus-rpc is NOT

- **Not a routing framework for HTTP REST.** This is JSON-RPC streaming over WebSocket / stdio / MCP HTTP. The optional REST gateway in `plexus-transport` is for adapting REST callers; the canonical surface is JSON-RPC.
- **Not a code generator for the server side.** Server code IS the schema. The codegen story is on the client side (TypeScript, Rust SDK consumers).
- **Not an auth provider.** The framework defines the *primitives* (`AuthContext`, `Principal`, `Credential<T>`, `Tenant`) and the wire envelopes; you bring your own JWT validator, OIDC client, or whatever — and it plugs in as a `SessionValidator` implementation.

## License

MIT. See `LICENSE`.

## Status

Pre-1.0. APIs are stable in shape but subject to additive change. Each subcrate carries its own changelog; the umbrella's `Capabilities.features` is the canonical "what shipped in this release" reference.
