use std::{borrow::Cow, path::{Path, PathBuf}, process::Command};
use helix_loader::repo_paths

const VERSION: &str = repo_paths::version().to_str()
    .expect("VERSION path should have valid UTF-8 encoding");

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "HEAD"]).output().ok()
        .filter(|output| output.status.success())
        .and_then(|x| String::from_utf8(x.stdout).ok());

    let version: Cow<_> = match git_hash {
        Some(git_hash) => format!("{} ({})", VERSION, &git_hash[..8]).into(),
        None => VERSION.into(),
    };

    println!("cargo:rustc-env=BUILD_TARGET={}",std::env::var("TARGET").unwrap());
    println!("cargo:rerun-if-changed={}", VERSION);
    println!("cargo:rustc-env=VERSION_AND_GIT_HASH={}", version);
}
