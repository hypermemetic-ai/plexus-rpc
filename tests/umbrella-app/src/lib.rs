//! PLEXUS-44: an app that depends on `plexus-rpc` alone.
//!
//! Every macro the umbrella carries is used here — `#[activation]` with its
//! `#[method]`s and its generated client and trait, `#[derive(PlexusRequest)]`,
//! `#[derive(HandleEnum)]`, `#[derive(Credentials)]` — so each expansion has
//! to find every name it emits through `::plexus_rpc`. The crate's manifest
//! lists nothing else that could supply one.

// The names `#[derive(Credentials)]` writes are the crate's own imports.
use plexus_rpc::auth_core::{AttachmentSite, Credential, CredentialFieldMarker, CredentialKind, CredentialScheme, HeaderName};
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

/// What `lookup` finds.
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
pub struct Thing {
    pub id: String,
}

/// Why `lookup` found nothing.
#[derive(
    Debug,
    Clone,
    PartialEq,
    plexus_rpc::__deps::serde::Serialize,
    plexus_rpc::__deps::serde::Deserialize,
    plexus_rpc::__deps::schemars::JsonSchema,
)]
#[serde(crate = "plexus_rpc::__deps::serde", tag = "type", rename_all = "snake_case")]
#[schemars(crate = "plexus_rpc::__deps::schemars")]
pub enum Missing {
    NotFound { id: String },
    Unreachable { why: String },
}

impl FromClientError for Pong {
    fn from_client_error(e: ClientError) -> Self {
        Pong { message: e.to_string() }
    }
}
impl FromClientError for Tick {
    fn from_client_error(_: ClientError) -> Self {
        Tick { n: 0 }
    }
}
impl FromClientError for Missing {
    fn from_client_error(e: ClientError) -> Self {
        Missing::Unreachable { why: e.to_string() }
    }
}

/// What the activation reads off the request.
#[derive(PlexusRequest)]
pub struct EchoRequest {
    #[from_header("origin")]
    pub origin: Option<String>,
}

/// The token `sign_in` hands back.
pub struct SessionToken(pub String);

/// A method's answer that carries a credential.
#[derive(Credentials)]
#[allow(dead_code)]
pub struct SignedIn {
    pub user: String,
    #[credential(kind = "bearer", attach_as = "header:authorization", scheme = "Bearer ")]
    pub session: Credential<SessionToken>,
}

/// Handles the activation hands out.
#[derive(Debug, Clone, HandleEnum)]
#[handle(plugin_id = "EchoActivation::PLUGIN_ID", version = "1.0.0")]
pub enum EchoHandle {
    #[handle(method = "said")]
    Said { id: String },
    #[handle(method = "ping")]
    Ping,
}

/// The activation.
#[derive(Debug, Clone, Copy, Default)]
pub struct EchoActivation;

/// Say things back.
#[activation(namespace = "echo", version = "1.0.0", request = EchoRequest)]
impl EchoActivation {
    /// Always answers pong.
    #[method]
    pub async fn ping(&self) -> Pong {
        Pong { message: "pong".into() }
    }

    /// Ticks from 1 up to `to`.
    #[method]
    pub async fn count(&self, /** the last tick */ to: u32) -> impl Stream<Item = Tick> + Send + 'static {
        plexus_rpc::__deps::futures::stream::iter((1..=to).map(|n| Tick { n }))
    }

    /// The thing with this id; "missing" is not found.
    #[method]
    pub async fn lookup(&self, id: String) -> Result<Thing, Missing> {
        if id == "missing" {
            Err(Missing::NotFound { id })
        } else {
            Ok(Thing { id })
        }
    }
}

/// The service: the activation, registered under a root that is not its
/// namespace.
pub fn service() -> DynamicService {
    DynamicService::new("umbrella").register(EchoActivation)
}
