use std::process::Command;

fn main() {
    let version = std::env::var("CCSM_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=CCSM_VERSION={version}");
    println!("cargo:rustc-env=GIT_COMMIT_SHORT={commit}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=CCSM_VERSION");
}
