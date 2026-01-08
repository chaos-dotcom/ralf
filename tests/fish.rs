use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::process::Command;
mod common;

#[test]
fn fish_can_use_generated_aliases() {
    // Skip if fish not available
    if which::which("fish").is_err() {
        eprintln!("fish not found; skipping fish test");
        return;
    }

    let temp = TempDir::new().unwrap();
    common::copy_fixture_to(&temp, "generate/alf.conf", "alf.conf");
    let aliases = temp.child("aliases.fish");

    // Generate aliases as Fish (force Fish detection for generator)
    common::cmd()
        .current_dir(temp.path())
        .env("SHELL", "fish")
        .env("ALF_ALIASES_FILE", aliases.path())
        .arg("save")
        .assert()
        .success();

    // Run a fish one-liner to source and execute "say again fish-works"
    let out = Command::new("fish")
        .arg("-lc")
        .arg(format!("source {}; say again fish-works", aliases.path().display()))
        .output()
        .unwrap();

    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "fish-works fish-works");
}
