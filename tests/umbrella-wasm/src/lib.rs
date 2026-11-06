//! PLEXUS-45: an activation crate on `plexus-rpc` alone, a service natively
//! and a client in wasm32. Every name the macro emits — in the client half
//! with the umbrella's `server` off — resolves through `::plexus_rpc`.

use plexus_rpc::prelude::*;

/// One answer.
#[derive(
    Debug,
    Clone,
    PartialEq,
    plexus_rpc::__deps::serde::Serialize,
    plexus_rpc::__deps::serde::Deserialize,
    plexus_rpc::__deps::schemars::JsonSchema,
)]
#[serde(crate = "plexus_rpc::__deps::serde")]
#[schemars(crate = "plexus_rpc::__deps::schemars")]
pub struct Pong {
    pub message: String,
}

impl FromClientError for Pong {
    fn from_client_error(error: ClientError) -> Self {
        Pong { message: error.to_string() }
    }
}

/// One of many.
#[derive(
    Debug,
    Clone,
    PartialEq,
    plexus_rpc::__deps::serde::Serialize,
    plexus_rpc::__deps::serde::Deserialize,
    plexus_rpc::__deps::schemars::JsonSchema,
)]
#[serde(crate = "plexus_rpc::__deps::serde")]
#[schemars(crate = "plexus_rpc::__deps::schemars")]
pub struct Tick {
    pub n: u32,
}

impl FromClientError for Tick {
    fn from_client_error(_: ClientError) -> Self {
        Tick { n: 0 }
    }
}

#[derive(Clone, Default)]
pub struct EchoActivation;

/// Say things back.
#[activation(namespace = "echo", version = "1.0.0")]
impl EchoActivation {
    /// One pong.
    #[method]
    pub async fn ping(&self) -> Pong {
        Pong { message: "pong".into() }
    }

    /// Ticks from 1 up to `to`.
    #[method]
    pub async fn count(&self, to: u32) -> impl Stream<Item = Tick> + Send + 'static {
        plexus_rpc::__deps::futures::stream::iter((1..=to).map(|n| Tick { n }))
    }
}

/// The service, natively.
#[cfg(feature = "server")]
pub fn service() -> DynamicService {
    DynamicService::new("umbrella-wasm").register(EchoActivation)
}

/// The generated client, on every target: what a browser crate calls.
pub async fn ping_through<T: Transport + Clone + Send + Sync + 'static>(transport: T) -> Pong {
    Echo::ping(&EchoClient::new(transport)).await
}
