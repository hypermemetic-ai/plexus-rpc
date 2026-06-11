//! Minimal Plexus RPC server.
//!
//! Run:
//!
//! ```bash
//! cargo run --example echo
//! ```
//!
//! Then, in another terminal:
//!
//! ```bash
//! synapse echo_hub echo echo --message "hello" --count 3
//! synapse echo_hub echo once --message "hi"
//! ```
//!
//! (`echo_hub` is the backend / hub root name; `echo` is the activation
//! namespace; the final segment is the method. The root name must differ
//! from every activation namespace — see Z2H-8.)
//!
//! The server binds to `127.0.0.1:4444` — the default port synapse looks
//! for when no `--port` flag is set, so the bare invocations above find
//! it automatically.

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

#[derive(Clone)]
pub struct Echo;

#[plexus_macros::activation(
    namespace = "echo",
    version = "1.0.0",
    description = "Echo messages back"
)]
impl Echo {
    /// Echo a message back the specified number of times.
    #[plexus_macros::method]
    async fn echo(
        &self,
        /// The message to echo
        message: String,
        /// Number of times to repeat
        count: u32,
    ) -> impl Stream<Item = EchoEvent> + Send + 'static {
        stream! {
            for i in 0..count {
                yield EchoEvent::Echo {
                    message: message.clone(),
                    count: i + 1,
                };
            }
        }
    }

    /// Echo a message once.
    #[plexus_macros::method]
    async fn once(
        &self,
        /// The message to echo
        message: String,
    ) -> impl Stream<Item = EchoEvent> + Send + 'static {
        stream! {
            yield EchoEvent::Echo { message, count: 1 };
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // The hub root name is a routing namespace, not a label: it must differ
    // from every registered activation's namespace, or registration panics
    // (a like-named child would be unreachable — Z2H-8). The activation
    // namespace ("echo") is what you type at the synapse CLI; the root name
    // never appears in everyday invocations.
    let hub = Arc::new(DynamicHub::new("echo_hub").register(Echo));

    let rpc_converter = |arc| {
        DynamicHub::arc_into_rpc_module(arc)
            .map_err(|e| anyhow::anyhow!("rpc module: {e}"))
    };

    println!("plexus-rpc echo example");
    println!("  ws://127.0.0.1:4444 — synapse will discover this on its default port");
    println!("  $ synapse echo_hub echo echo --message hello --count 3");
    println!("  Ctrl-C to stop.");

    TransportServer::builder(hub, rpc_converter)
        .with_websocket(4444)
        .build()
        .await?
        .serve()
        .await
}
