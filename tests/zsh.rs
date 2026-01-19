use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::process::Command;
mod common;

#[test]
fn zsh_can_use_generated_aliases() {
    // Skip if zsh not available
    if which::which("zsh").is_err() {
        eprintln!("zsh not found; skipping zsh test");
        return;
    }

    let temp = TempDir::new().unwrap();
    common::copy_fixture_to(&temp, "generate/alf.conf", "alf.conf");
    let aliases = temp.child("aliases.txt");

    // Generate aliases
    common::cmd()
        .current_dir(temp.path())
        .env("ALF_ALIASES_FILE", aliases.path())
        .arg("save")
        .assert()
        .success();

    // Run a zsh one-liner to source and execute "say again zsh-works"
    let out = Command::new("zsh")
        .arg("-lc")
        .arg(format!(
            "source {}; say again zsh-works",
            aliases.path().display()
        ))
        .output()
        .unwrap();

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "zsh-works zsh-works"
    );
}
