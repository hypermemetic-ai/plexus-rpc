//! PLEXUS-45: with `default-features = false` the umbrella — plexus-macros,
//! plexus-client, plexus-auth-core and `__deps` — builds for
//! `wasm32-unknown-unknown`, and so does an activation crate on it with its
//! own `server` off (`tests/umbrella-wasm`: the trait and the generated
//! client). Skipped, with a message, when the target is not installed.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::Command;

const TARGET: &str = "wasm32-unknown-unknown";

fn target_installed() -> bool {
    let sysroot = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .args(["--print", "sysroot"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));
    sysroot.is_some_and(|s| s.join("lib/rustlib").join(TARGET).exists())
}

fn check(package: &str) {
    // A target and build directory of its own: the suite's cargo holds its own.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("wasm32-check");
    let out = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["check", "-p", package, "--lib", "--no-default-features", "--target", TARGET, "--message-format", "short"])
        .env("CARGO_TARGET_DIR", &dir)
        .env("CARGO_BUILD_BUILD_DIR", &dir)
        // the host's flags are not the target's; getrandom 0.3 takes its
        // randomness from JS only under this cfg
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS", r#"--cfg getrandom_backend="wasm_js""#)
        .output()
        .expect("cargo runs");
    assert!(
        out.status.success(),
        "cargo check -p {package} --lib --no-default-features --target {TARGET} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn without_the_server_the_umbrella_and_a_client_crate_build_for_wasm32() {
    if !target_installed() {
        eprintln!("SKIPPED: {TARGET} is not installed (rustup target add {TARGET}); the wasm32 build is unchecked");
        return;
    }
    check("plexus-rpc");
    check("umbrella-wasm");
}
