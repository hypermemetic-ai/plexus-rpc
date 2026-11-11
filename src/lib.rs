//! # Plexus RPC — one dependency for a service
//!
//! ```toml
//! [dependencies]
//! plexus-rpc = "0.11"
//! ```
//!
//! That line brings the set at versions that work together, its version in
//! lockstep with plexus-core's:
//!
//! - [`core`] (feature `server`, on by default) — `DynamicHub`, dispatch, schemas, the
//!   `Activation` trait (`plexus-core`);
//! - [`macros`] — `#[activation]`, `#[method]`, `#[child]` and the derives (`plexus-macros`),
//!   the attribute macros also at this crate's root;
//! - [`auth_core`] — the sealed identity, credential, grant and audit types (`plexus-auth-core`);
//! - [`client`] — the runtime of the client an activation's macro writes (`plexus-client`);
//! - [`transport`] (feature `transport`, on by default) — the WebSocket / HTTP / stdio
//!   server (`plexus-transport`);
//! - [`prelude`] — what an activation and its `main` usually name.
//!
//! The macros' expansions name `async_trait`, `futures`, `jsonrpsee`,
//! `schemars`, `serde`, `serde_json`, `tokio` and `uuid`. With this crate a
//! dependency, plexus-macros routes every one of them through
//! [`__deps`], so an app lists none of them — only what its own code uses.
//!
//! ```ignore
//! use plexus_rpc::prelude::*;
//!
//! #[derive(Clone)]
//! pub struct EchoActivation;
//!
//! /// Say things back.
//! #[activation(namespace = "echo", version = "1.0.0")]
//! impl EchoActivation {
//!     /// Say a thing back.
//!     #[method]
//!     async fn say(
//!         &self,
//!         /// what to say
//!         text: String,
//!     ) -> String {
//!         text
//!     }
//! }
//! ```
//!
//! A type that crosses the wire derives `PlexusWire` (PLEXUS-59): serde's
//! `Serialize` and `Deserialize` and schemars' `JsonSchema`, and for an enum
//! the tag every Plexus event carries (`tag = "type"`, snake_case):
//!
//! ```ignore
//! #[derive(Debug, Clone, PlexusWire)]
//! pub struct Tick {
//!     n: u32,
//! }
//! ```
//!
//! A type that derives serde or schemars traits itself names them
//! through `__deps` when the app does not list serde itself:
//! `#[serde(crate = "plexus_rpc::__deps::serde")]`,
//! `#[schemars(crate = "plexus_rpc::__deps::schemars")]`.
//!
//! ## Capability manifest
//!
//! Backends embed [`CAPABILITIES`] in their `_info` response so generic clients
//! (synapse, generated SDKs, agents) can negotiate features instead of
//! guessing from version strings.

/// Plexus RPC umbrella crate version, populated at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The plexus-macros version this umbrella requires (its `Cargo.toml`
/// pin). plexus-macros is a proc-macro crate and cannot export a `VERSION`
/// of its own.
pub const PLEXUS_MACROS_VERSION: &str = "0.11.2";

// ---------------------------------------------------------------------------
// The crates.
// ---------------------------------------------------------------------------

/// Sealed identity, credential, tenant, grant, forwarding and audit
/// primitives — the `plexus-auth-core` crate.
pub use plexus_auth_core as auth_core;

/// Dispatch, schemas, the `Activation` trait and `DynamicHub` — the
/// `plexus-core` crate (feature `server`, on by default).
#[cfg(feature = "server")]
pub use plexus_core as core;

/// The procedural macros — the `plexus-macros` crate. The attribute
/// macros are also at this crate's root (`plexus_rpc::activation`).
pub use plexus_macros as macros;

/// The runtime of the client `#[activation]` writes: `Transport`, the
/// item stream, errors and (feature `client-ws`) `ws::WsTransport` — the
/// `plexus-client` crate.
pub use plexus_client as client;

/// The WebSocket / HTTP / stdio server runtime (feature `transport`, on by
/// default) — the `plexus-transport` crate.
#[cfg(feature = "transport")]
pub use plexus_transport as transport;

// The attribute macros, at the root.
#[allow(deprecated)]
pub use plexus_macros::{activation, child, method, peer, removed_in, PlexusWire};
/// `PlexusWire` under the name it has behind the crate's own: `#[derive(plexus::Wire)]`.
pub use plexus_macros::PlexusWire as Wire;

/// What an activation and the `main` that serves it usually name.
///
/// ```ignore
/// use plexus_rpc::prelude::*;
/// ```
pub mod prelude {
    pub use plexus_macros::{activation, child, method, peer, removed_in, Credentials, HandleEnum, PlexusRequest, PlexusWire};

    #[cfg(feature = "server")]
    pub use plexus_core::plexus::{
        Activation, AuthContext, ChildRouter, DynamicHub, HubTransport, Input, Interface, Peer, PeerError, PlexusError,
        PlexusStream, PluginSchema, Progress,
    };
    /// The service an app registers its activations with and serves:
    /// plexus-core's `DynamicHub`, by the name the ecosystem uses for it.
    #[cfg(feature = "server")]
    pub use plexus_core::plexus::DynamicHub as DynamicService;
    /// A service called in process through its own routing: plexus-core's
    /// `HubTransport`, by the name the ecosystem uses for it.
    #[cfg(feature = "server")]
    pub use plexus_core::plexus::HubTransport as ServiceTransport;

    pub use plexus_client::{BoxFuture, BoxStream, ClientError, FromClientError, RemoteError, Transport};

    #[cfg(feature = "transport")]
    pub use plexus_transport::TransportServer;

    pub use futures::{Stream, StreamExt};
}

// ---------------------------------------------------------------------------
// The crates the macros' expansions name (Z2H-2).
// ---------------------------------------------------------------------------

/// Third-party crates referenced by `plexus-macros` expansions, re-exported
/// so that an app depending on `plexus-rpc` needs none of them in its own
/// Cargo.toml.
///
/// When plexus-macros finds `plexus-rpc` in the app's manifest (via
/// `proc-macro-crate`), its expansions resolve `async_trait`, `futures`,
/// `jsonrpsee`, `schemars`, `serde`, `serde_json`, `tokio` and `uuid`
/// through this module. `anyhow` and `async_stream` are here for the app:
/// the usual activation writes them, no macro emits them.
///
/// Hidden because it is macro plumbing, not API; the one place an app names
/// it is a derive's crate attribute on a type of its own, when it does not
/// list serde or schemars itself:
/// `#[serde(crate = "plexus_rpc::__deps::serde")]`,
/// `#[schemars(crate = "plexus_rpc::__deps::schemars")]`.
#[doc(hidden)]
pub mod __deps {
    pub use ::anyhow;
    pub use ::async_stream;
    pub use ::async_trait;
    pub use ::futures;
    #[cfg(feature = "server")]
    pub use ::jsonrpsee;
    pub use ::schemars;
    pub use ::serde;
    pub use ::serde_json;
    pub use ::tokio;
    pub use ::uuid;
}

// ---------------------------------------------------------------------------
// Capability manifest (UMB-3).
// ---------------------------------------------------------------------------

/// Compile-time capability manifest describing the subcrate versions and
/// feature flags bundled in this release of `plexus-rpc`.
///
/// Backends embed [`CAPABILITIES`] in their `_info` response. Synapse +
/// synapse-cc decode it and branch on `features` rather than version
/// strings, so adding a new feature is one entry on this struct and one
/// branch in tooling — no version-string parsing involved.
///
/// `Capabilities` is **emit-only** — it holds `&'static` slices so it can
/// be a `const`, which precludes `Deserialize`. Consumers should mirror
/// the wire shape in their own owning struct.
#[cfg(feature = "server")]
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Capabilities {
    /// Version of the `plexus-rpc` umbrella itself.
    pub plexus_rpc: &'static str,
    /// Bundled `plexus-auth-core` version.
    pub plexus_auth_core: &'static str,
    /// Bundled `plexus-core` version.
    pub plexus_core: &'static str,
    /// Bundled `plexus-macros` version ([`PLEXUS_MACROS_VERSION`], the
    /// umbrella's pin).
    pub plexus_macros: &'static str,
    /// Bundled `plexus-transport` version. `None` when the umbrella was
    /// built without the `transport` feature.
    pub plexus_transport: Option<&'static str>,
    /// Named feature flags shipped in this release. Tooling branches on
    /// these instead of parsing version strings.
    pub features: &'static [&'static str],
}

/// The canonical compile-time capability manifest for this build of
/// `plexus-rpc`.
///
/// Backends embed this in their `_info` payload so tooling (synapse,
/// synapse-cc, generated SDKs) can branch on capability rather than
/// inferring features from subcrate version strings. A service's (feature
/// `server`).
#[cfg(feature = "server")]
pub const CAPABILITIES: Capabilities = Capabilities {
    plexus_rpc: VERSION,
    plexus_auth_core: plexus_auth_core::VERSION,
    plexus_core: plexus_core::VERSION,
    plexus_macros: PLEXUS_MACROS_VERSION,
    #[cfg(feature = "transport")]
    plexus_transport: Some(plexus_transport::VERSION),
    #[cfg(not(feature = "transport"))]
    plexus_transport: None,
    features: FEATURES,
};

/// The named feature flags this release of `plexus-rpc` advertises. Tooling
/// (synapse, synapse-cc, generated SDKs) branches on these strings rather
/// than parsing version numbers. New features get added to this list as
/// they land.
///
/// Current entries reflect wave 2 (May 2026) of the AUTHZ epic:
///
/// - `"auth_capabilities"` — backend advertises auth mechanisms at `_info`
///   (AUTHZ-CORE-3).
/// - `"forward_policy"` — per-namespace `ForwardPolicy` consulted on every
///   cross-boundary call (AUTHLANG-2 + AUTHLANG-3).
/// - `"credentials"` — `Credential<T>` wire envelope (sentinel + sidecar)
///   with `MethodSchema` projection (AUTHZ-CRED-CORE-1, -2, -3 +
///   `#[derive(Credentials)]`).
/// - `"tenant"` — sealed `Tenant` identity + `ClaimTenantResolver` +
///   `Tenanted<S>` storage wrapper (AUTHZ-DATA-1-TYPES + WRAPPER).
/// - `"audit"` — `AuditRecord` + `AuditSink` primitives + `TracingAuditSink`
///   default (AUTHZ-PRIVACY-1).
/// - `"doc_comments"` — `///` on function and parameters feeds the schema
///   (AUTHZ-MACRO-PARAM-DOC-1).
/// - `"optional_auth"` — `auth: Option<&AuthContext>` codegen support
///   (AUTHZ-MACRO-OPTIONAL-AUTH-1).
/// - `"unified_schema"` — PROT schema unification (PLX-13): every `.schema`
///   fetch at any tree depth returns the single `PluginSchema` (`SchemaResult`
///   is `Plugin`-only). The former per-method `SchemaResult::Method` variant,
///   the `{method}.schema` strip-suffix shortcut, and the `.method_schema`
///   content-type no longer exist — per-method detail is read from `methods[]`.
///   Tooling can rely on `".schema" isSuffixOf content_type` always matching.
#[cfg(feature = "server")]
const FEATURES: &[&str] = &[
    "auth_capabilities",
    "forward_policy",
    "credentials",
    "tenant",
    "audit",
    "doc_comments",
    "optional_auth",
    "unified_schema",
];

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn version_const_is_populated() {
        assert!(!VERSION.is_empty());
        // Must match what's pinned in our Cargo.toml.
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn capabilities_const_reflects_subcrate_versions() {
        assert_eq!(CAPABILITIES.plexus_rpc, VERSION);
        assert_eq!(CAPABILITIES.plexus_auth_core, plexus_auth_core::VERSION);
        assert_eq!(CAPABILITIES.plexus_core, plexus_core::VERSION);
        assert_eq!(CAPABILITIES.plexus_macros, PLEXUS_MACROS_VERSION);
        assert!(!CAPABILITIES.features.is_empty());
    }

    /// The constant is the manifest's pin, so the manifest says what was
    /// built against. (Run in the repository; a published crate's manifest
    /// is cargo's normalized one.)
    #[test]
    fn plexus_macros_version_is_the_manifest_pin() {
        let manifest = include_str!("../Cargo.toml");
        let lines: Vec<&str> = manifest.lines().map(str::trim).collect();
        let pin = format!("plexus-macros = \"{}\"", PLEXUS_MACROS_VERSION);
        // cargo's normalized manifest: a `[dependencies.plexus-macros]` table.
        let table = lines.windows(2).any(|w| {
            w[0] == "[dependencies.plexus-macros]" && w[1] == format!("version = \"{}\"", PLEXUS_MACROS_VERSION)
        });
        assert!(lines.contains(&pin.as_str()) || table, "Cargo.toml must pin {pin}");
    }

    /// In lockstep: the umbrella's major.minor is plexus-core's.
    #[test]
    fn version_is_in_lockstep_with_plexus_core() {
        let minor = |v: &str| v.split('.').take(2).collect::<Vec<_>>().join(".");
        assert_eq!(minor(VERSION), minor(plexus_core::VERSION));
    }

    #[test]
    fn capabilities_serializes_to_expected_json_keys() {
        // Capabilities is emit-only (const, &'static slices). Verify the
        // wire shape carries every field a consumer would expect — they
        // mirror it into an owning struct of their own.
        let v = serde_json::to_value(CAPABILITIES).expect("capabilities serialize");
        for key in [
            "plexus_rpc",
            "plexus_auth_core",
            "plexus_core",
            "plexus_macros",
            "plexus_transport",
            "features",
        ] {
            assert!(
                v.get(key).is_some(),
                "wire shape must carry key {} so consumers can mirror it",
                key
            );
        }
    }

    #[test]
    #[cfg(feature = "transport")]
    fn transport_version_is_present_when_feature_is_on() {
        assert!(CAPABILITIES.plexus_transport.is_some());
        assert_eq!(
            CAPABILITIES.plexus_transport,
            Some(plexus_transport::VERSION)
        );
    }

    #[test]
    #[cfg(not(feature = "transport"))]
    fn transport_version_is_absent_when_feature_is_off() {
        assert!(CAPABILITIES.plexus_transport.is_none());
    }

    #[test]
    fn features_list_is_stable() {
        // Document the v0.1 feature set so a casual reader of the test file
        // sees what shipped. New features get appended here as they land.
        for required in [
            "auth_capabilities",
            "forward_policy",
            "credentials",
            "tenant",
            "audit",
            "doc_comments",
            "optional_auth",
        ] {
            assert!(
                CAPABILITIES.features.contains(&required),
                "feature flag {} missing from CAPABILITIES.features",
                required
            );
        }
    }
}
