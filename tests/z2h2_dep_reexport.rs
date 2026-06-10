//! Z2H-2 ANTI-ROT GATE — umbrella-qualified macro emission must not regress.
//!
//! plexus-macros 0.6 emits dependency paths through `plexus_rpc::__deps`
//! when the consumer's Cargo.toml lists `plexus-rpc` (detected via
//! proc-macro-crate), and today's bare extern-prelude paths otherwise. This
//! test scaffolds BOTH consumer shapes into a tempdir and builds them
//! against the real sibling checkouts — the same matrix the Z2H-S2 spike
//! proved:
//!
//! 1. umbrella consumer: Cargo.toml lists ONLY `plexus-rpc` + `jsonrpsee`.
//!    The 12-dep leak Z2H-R2 measured is dead. (`jsonrpsee` stays direct
//!    because its own `#[rpc]` proc-macro looks itself up in the consumer's
//!    manifest — see plexus-rpc/Cargo.toml; true 1-dep is a noted follow-up.)
//! 2. direct consumer: the pre-Z2H-2 sibling-checkout shape with explicit
//!    deps and NO `plexus-rpc`. Compiles unchanged — no regression.
//!
//! Layout: the sibling repos are copied into a tempdir "farm" so that every
//! `path = "../x"` hop resolves inside the farm. (Symlinks don't work:
//! cargo canonicalizes path-dep locations before resolving their own
//! relative deps.) Override `PLEXUS_MACROS_DIR` to test an out-of-tree
//! plexus-macros checkout (e.g. a feature worktree); it defaults to the
//! `../plexus-macros` sibling.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const UMBRELLA_MAIN_RS: &str = r##"
//! Umbrella-only consumer: every macro-emitted dep resolves through
//! plexus_rpc::__deps. User-side derives use the crate= escape hatches.
use plexus_rpc::__deps::{futures, tokio};
use plexus_rpc::macros::{activation, PlexusRequest};

#[derive(Debug, Clone, plexus_rpc::__deps::serde::Serialize, plexus_rpc::__deps::schemars::JsonSchema)]
#[serde(tag = "event", rename_all = "snake_case")]
#[serde(crate = "plexus_rpc::__deps::serde")]
#[schemars(crate = "plexus_rpc::__deps::schemars")]
pub enum EchoEvent {
    Echo { message: String },
    Done { count: u32 },
}

#[derive(PlexusRequest)]
pub struct EchoRequest {
    #[from_header("origin")]
    pub origin: Option<String>,
}

#[derive(Clone)]
pub struct Echo;

/// Echo backend — umbrella-only consumer proof.
#[activation(
    namespace = "echo",
    version = "1.0.0",
    description = "Echo a message back",
    request = EchoRequest,
)]
impl Echo {
    /// Echo a message a number of times.
    #[method]
    async fn echo(
        &self,
        /// The message to echo
        message: String,
        /// Number of repetitions
        count: u32,
    ) -> impl futures::Stream<Item = EchoEvent> + Send + 'static {
        futures::stream::iter(
            (0..count)
                .map(move |_| EchoEvent::Echo { message: message.clone() })
                .chain(std::iter::once(EchoEvent::Done { count })),
        )
    }
}

fn main() {
    // NB: #[tokio::main] is unusable umbrella-only (tokio's macro emits
    // bare `tokio` paths); the Builder form is the documented shape.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");
    rt.block_on(async {
        use plexus_rpc::core::plexus::{Activation, DynamicHub};
        let echo = Echo;
        assert_eq!(echo.namespace(), "echo");
        assert_eq!(Echo::PLUGIN_ID, Echo.plugin_id());
        let schema = echo.plugin_schema();
        assert!(schema.methods.iter().any(|m| m.name == "echo"));
        let _hub = DynamicHub::new("echo").register(Echo);
        println!("umbrella-only consumer green");
    });
}
"##;

const DIRECT_MAIN_RS: &str = r##"
//! Direct-deps consumer: today's sibling-checkout shape, unchanged.
//! Plain derives, bare crate names, #[tokio::main] — nothing umbrella-aware.
use async_stream::stream;
use futures::Stream;
use plexus_core::plexus::DynamicHub;
use plexus_macros::{activation, PlexusRequest};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EchoEvent {
    Echo { message: String },
    Done { count: u32 },
}

#[derive(PlexusRequest)]
pub struct EchoRequest {
    #[from_header("origin")]
    pub origin: Option<String>,
}

#[derive(Clone)]
pub struct Echo;

/// Echo backend — direct-deps consumer control.
#[activation(
    namespace = "echo",
    version = "1.0.0",
    description = "Echo a message back",
    request = EchoRequest,
)]
impl Echo {
    /// Echo a message a number of times.
    #[method]
    async fn echo(
        &self,
        /// The message to echo
        message: String,
        /// Number of repetitions
        count: u32,
    ) -> impl Stream<Item = EchoEvent> + Send + 'static {
        stream! {
            for _ in 0..count {
                yield EchoEvent::Echo { message: message.clone() };
            }
            yield EchoEvent::Done { count };
        }
    }
}

#[tokio::main]
async fn main() {
    use plexus_core::plexus::Activation;
    let echo = Echo;
    assert_eq!(echo.namespace(), "echo");
    assert_eq!(Echo::PLUGIN_ID, Echo.plugin_id());
    let schema = echo.plugin_schema();
    assert!(schema.methods.iter().any(|m| m.name == "echo"));
    let _hub = DynamicHub::new("echo").register(Echo);
    println!("direct-deps consumer green");
}
"##;

/// The names Z2H-R2 measured leaking into consumer manifests. The umbrella
/// consumer's direct dependency list must contain none of them.
const LEAKED_NAMES: &[&str] = &[
    "plexus-macros",
    "plexus-core",
    "anyhow",
    "futures",
    "async-stream",
    "tokio",
    "serde",
    "schemars",
    "uuid",
    "async-trait",
    "serde_json",
];

/// Recursively copy a crate directory, skipping build artifacts and VCS.
fn copy_crate(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("mkdir");
    for entry in fs::read_dir(src).expect("read_dir") {
        let entry = entry.expect("dir entry");
        let name = entry.file_name();
        if name == "target" || name == ".git" {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        if entry.file_type().expect("file_type").is_dir() {
            copy_crate(&from, &to);
        } else {
            fs::copy(&from, &to).expect("copy file");
        }
    }
}

fn cargo(args: &[&str], dir: &Path, target_dir: &Path) -> std::process::Output {
    Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("failed to spawn cargo")
}

#[test]
fn umbrella_and_direct_consumers_compile() {
    let rpc_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let siblings = rpc_dir.parent().expect("siblings root").to_path_buf();
    let macros_dir = std::env::var_os("PLEXUS_MACROS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| siblings.join("plexus-macros"));

    let tmp = tempfile::tempdir().expect("tempdir");
    let farm = tmp.path();
    let target_dir = farm.join("target");

    // 1. Farm: copies so relative path-deps resolve inside the farm.
    copy_crate(&rpc_dir, &farm.join("plexus-rpc"));
    copy_crate(&macros_dir, &farm.join("plexus-macros"));
    for sibling in [
        "plexus-core",
        "plexus-auth-core",
        "plexus-transport",
        "plexus-schemars-compat",
    ] {
        copy_crate(&siblings.join(sibling), &farm.join(sibling));
    }

    // 2. Umbrella consumer — TWO deps, nothing else.
    let umbrella = farm.join("consumer-umbrella");
    fs::create_dir_all(umbrella.join("src")).expect("mkdir umbrella");
    fs::write(
        umbrella.join("Cargo.toml"),
        r#"[package]
name = "consumer-umbrella"
version = "0.1.0"
edition = "2021"

[dependencies]
plexus-rpc = { path = "../plexus-rpc" }
jsonrpsee = { version = "0.26", features = ["server", "macros"] }
"#,
    )
    .expect("write umbrella manifest");
    fs::write(umbrella.join("src/main.rs"), UMBRELLA_MAIN_RS).expect("write umbrella main");

    let out = cargo(&["build"], &umbrella, &target_dir);
    assert!(
        out.status.success(),
        "Z2H-2 AC #1/#2 FAILED: umbrella-only consumer (plexus-rpc + jsonrpsee) did not compile:\n{}",
        String::from_utf8_lossy(&out.stderr),
    );

    // 3. Leak-death evidence: the umbrella consumer's DIRECT dependency
    //    list is exactly {jsonrpsee, plexus-rpc}.
    let out = cargo(
        &["tree", "-e", "normal", "--depth", "1", "--prefix", "none"],
        &umbrella,
        &target_dir,
    );
    assert!(out.status.success(), "cargo tree failed");
    let tree = String::from_utf8_lossy(&out.stdout);
    let direct: Vec<&str> = tree
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|n| !n.is_empty() && *n != "consumer-umbrella")
        .collect();
    assert_eq!(
        direct,
        vec!["jsonrpsee", "plexus-rpc"],
        "umbrella consumer's direct deps changed:\n{tree}"
    );
    for leaked in LEAKED_NAMES {
        assert!(
            !direct.contains(leaked),
            "Z2H-R2 leak resurfaced: {leaked} is a direct dependency again:\n{tree}"
        );
    }

    // 4. Direct consumer — the pre-Z2H-2 shape compiles unchanged.
    let direct_dir = farm.join("consumer-direct");
    fs::create_dir_all(direct_dir.join("src")).expect("mkdir direct");
    fs::write(
        direct_dir.join("Cargo.toml"),
        r#"[package]
name = "consumer-direct"
version = "0.1.0"
edition = "2021"

[dependencies]
plexus-core = { path = "../plexus-core" }
plexus-macros = { path = "../plexus-macros" }
anyhow = "1.0"
async-stream = "0.3"
async-trait = "0.1"
futures = "0.3"
jsonrpsee = { version = "0.26", features = ["server", "macros"] }
schemars = { version = "1.1", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
"#,
    )
    .expect("write direct manifest");
    fs::write(direct_dir.join("src/main.rs"), DIRECT_MAIN_RS).expect("write direct main");

    let out = cargo(&["build"], &direct_dir, &target_dir);
    assert!(
        out.status.success(),
        "Z2H-2 AC #3 FAILED: direct-deps consumer regressed:\n{}",
        String::from_utf8_lossy(&out.stderr),
    );
}
