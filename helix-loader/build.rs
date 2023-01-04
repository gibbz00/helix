use std::{borrow::Cow, process::Command};

const VERSION_PATH: &str = "VERSION";

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "HEAD"]).output().ok()
        .filter(|output| output.status.success())
        .and_then(|x| String::from_utf8(x.stdout).ok());

    let version: Cow<_> = match git_hash {
        Some(git_hash) => format!("{} ({})", VERSION_PATH, &git_hash[..8]).into(),
        None => VERSION_PATH.into(),
    };

    println!("cargo:rerun-if-changed={}", VERSION_PATH);
    println!("cargo:rustc-env=VERSION_AND_GIT_HASH={}", version);
}
