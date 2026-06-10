//! # Plexus RPC — umbrella crate
//!
//! The single dependency to add when building a Plexus backend. Re-exports
//! the verified-compatible subcrate set:
//!
//! - [`auth_core`] — sealed identity, credential, tenant, forwarding, and audit
//!   primitives (`plexus-auth-core`).
//! - [`core`] — `DynamicHub`, dispatch, `MethodSchema`, the `Activation` trait,
//!   credential wire envelope, ChildRouter (`plexus-core`).
//! - [`macros`] — procedural macros for activations, methods, child routers,
//!   credentials, and auth resolvers (`plexus-macros`).
//! - [`transport`] (feature `transport`, on by default) — WebSocket / HTTP / SSE
//!   server runtime (`plexus-transport`).
//!
//! ## Why one crate
//!
//! Tracking three or four subcrate version bumps across coordinated releases is
//! error-prone. Depending on `plexus-rpc` pins the whole set at once:
//!
//! ```toml
//! [dependencies]
//! plexus-rpc = "0.1"
//! ```
//!
//! ## Capability manifest
//!
//! Backends embed [`CAPABILITIES`] in their `_info` response so generic clients
//! (synapse CLI, generated SDKs, agents) can negotiate features instead of
//! guessing from version strings. The manifest names every bundled subcrate
//! version plus a list of named feature flags shipped in this release. See
//! `plans/UMB/UMB-3.md` for the full design.
//!
//! ## Doc-comment example (current canonical shape)
//!
//! ```ignore
//! use plexus_rpc::macros::{activation, method};
//!
//! pub struct Echo;
//!
//! #[activation(namespace = "echo", version = "1.0.0")]
//! impl Echo {
//!     /// Echo a message back the specified number of times.
//!     #[method]
//!     async fn echo(
//!         &self,
//!         /// The message to echo
//!         message: String,
//!         /// Number of times to repeat
//!         count: u32,
//!     ) -> impl futures::Stream<Item = EchoEvent> + Send + 'static {
//!         async_stream::stream! {
//!             for _ in 0..count {
//!                 yield EchoEvent::Echo { message: message.clone() };
//!             }
//!         }
//!     }
//! }
//! ```

/// Plexus RPC umbrella crate version, populated at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Subcrate re-exports.
// ---------------------------------------------------------------------------

/// Sealed identity, credential, tenant, forwarding, and audit primitives.
///
/// Re-export of the `plexus-auth-core` crate. The types here are
/// structurally sealed — third crates cannot fabricate `AuthContext`,
/// `Principal`, `Credential<T>`, `Tenant`, etc. Construction is the
/// framework's responsibility; activation code only ever receives sealed
/// references.
pub use plexus_auth_core as auth_core;

/// Core dispatch and schema machinery.
///
/// Re-export of the `plexus-core` crate. Contains `DynamicHub`,
/// `Activation`, `MethodSchema`, the credential wire envelope, the
/// `ChildRouter` trait, and the `with_auth_capabilities` / `with_forward_policy`
/// hub builders.
pub use plexus_core as core;

/// Procedural macros for activations, methods, child routers, credentials,
/// and auth resolvers.
///
/// Re-export of the `plexus-macros` crate. The canonical entry points are
/// `#[plexus_rpc::macros::activation(...)]` on an `impl` block,
/// `#[plexus_rpc::macros::method]` on each method, and
/// `#[derive(plexus_rpc::macros::Credentials)]` on credential-bearing
/// return types.
pub use plexus_macros as macros;

/// WebSocket / HTTP / SSE server runtime (feature `transport`, default on).
///
/// Re-export of the `plexus-transport` crate. Drop the `transport` feature
/// when building embedded / ahead-of-time codegen / WASM consumers that
/// only need the type and dispatch surface.
#[cfg(feature = "transport")]
pub use plexus_transport as transport;

// ---------------------------------------------------------------------------
// Macro-emission dependency re-exports (Z2H-2).
// ---------------------------------------------------------------------------

/// Third-party crates referenced by `plexus-macros` expansions, re-exported
/// so that umbrella consumers don't need them in their own Cargo.toml.
///
/// When `plexus-macros` 0.6 detects `plexus-rpc` in the consumer's manifest
/// (via `proc-macro-crate`), its expansions resolve `jsonrpsee`, `uuid`,
/// `async_trait`, `serde`, `serde_json`, `schemars`, `futures`, and `tokio`
/// through this module instead of the consumer's extern prelude. `anyhow`
/// and `async_stream` are included for consumer ergonomics (the canonical
/// activation examples use them) even though no macro emits them.
///
/// Hidden because it is macro plumbing, not API: user code should import
/// these crates directly (or via their own dependency declarations) rather
/// than reaching through `plexus_rpc::__deps`. Exception: derive attributes
/// on user event types in umbrella-only crates, e.g.
/// `#[serde(crate = "plexus_rpc::__deps::serde")]` /
/// `#[schemars(crate = "plexus_rpc::__deps::schemars")]`.
#[doc(hidden)]
pub mod __deps {
    pub use ::anyhow;
    pub use ::async_stream;
    pub use ::async_trait;
    pub use ::futures;
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
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Capabilities {
    /// Version of the `plexus-rpc` umbrella itself.
    pub plexus_rpc: &'static str,
    /// Bundled `plexus-auth-core` version.
    pub plexus_auth_core: &'static str,
    /// Bundled `plexus-core` version.
    pub plexus_core: &'static str,
    /// Bundled `plexus-macros` version. `plexus-macros` is a proc-macro
    /// crate and cannot expose a `pub const VERSION`, so the umbrella's
    /// build script reads the dependency pin from `Cargo.toml` and
    /// populates this at compile time.
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
/// inferring features from subcrate version strings.
pub const CAPABILITIES: Capabilities = Capabilities {
    plexus_rpc: VERSION,
    plexus_auth_core: plexus_auth_core::VERSION,
    plexus_core: plexus_core::VERSION,
    plexus_macros: env!("PLEXUS_MACROS_VERSION"),
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
const FEATURES: &[&str] = &[
    "auth_capabilities",
    "forward_policy",
    "credentials",
    "tenant",
    "audit",
    "doc_comments",
    "optional_auth",
];

#[cfg(test)]
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
        assert_eq!(CAPABILITIES.plexus_macros, env!("PLEXUS_MACROS_VERSION"));
        assert!(!CAPABILITIES.features.is_empty());
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
