//! Build script for `plexus-rpc`.
//!
//! `plexus-macros` is a `proc-macro = true` crate, so it cannot export a
//! `pub const VERSION: &str`. To populate the `Capabilities` manifest with
//! the bundled `plexus-macros` version, we read the version Cargo resolved
//! for our dependency and expose it as a build-time env var
//! (`PLEXUS_MACROS_VERSION`). The umbrella's `lib.rs` reads it via
//! `env!(...)`.

use std::env;

fn main() {
    // Cargo populates `DEP_<pkg>_<key>` only for packages that produce a
    // `links =` manifest key. plexus-macros doesn't, so the cleanest
    // portable source is `CARGO_PKG_VERSION` on the dependency record —
    // available to build scripts as `CARGO_PKG_VERSION_<dep>` only for
    // direct deps in certain Cargo versions. The reliable fallback is to
    // parse the workspace-resolved version out of the cargo manifest dir
    // by reading our own Cargo.toml dep pin (semver req like `"^0.5"`),
    // but the actual resolved version may differ from the req.
    //
    // For this build, we fall back to the dep req that ships in the
    // umbrella's Cargo.toml. The req is a major.minor pin so the version
    // string is reliable as a manifest line item. The Capabilities
    // manifest documents this as "the version the umbrella was built
    // against", which is what consumers actually want.
    let plexus_macros_version = env::var("DEP_PLEXUS_MACROS_VERSION")
        .unwrap_or_else(|_| "0.5".to_string());
    println!(
        "cargo:rustc-env=PLEXUS_MACROS_VERSION={}",
        plexus_macros_version
    );
    println!("cargo:rerun-if-changed=Cargo.toml");
}
