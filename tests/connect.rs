use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;
mod common;

#[test]
fn connect_https_username_noninteractive() {
    let temp = TempDir::new().unwrap();
    let bin_dir = temp.child("bin");
    common::write_fake_git(bin_dir.path());
    let alfrc = temp.child("alfrc");
    let aliases = temp.child("aliases.txt");

    common::cmd()
        .current_dir(temp.path())
        .env("PATH", common::prepend_to_path(bin_dir.path()))
        .env("ALF_RC_FILE", alfrc.path())
        .env("ALF_ALIASES_FILE", aliases.path())
        .args(["connect", "DannyBen", "--https"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Connecting to https://github.com/DannyBen/ralf-conf.git"))
        .stdout(predicate::str::contains("Storing location in"))
        .stdout(predicate::str::contains("Saving to"));

    let rc_content = fs::read_to_string(alfrc.path()).unwrap();
    assert!(rc_content.ends_with("/ralf-conf") || rc_content.ends_with("\\ralf-conf"));
    aliases.assert(predicates::path::exists());
}

#[test]
fn connect_full_url_yes() {
    let temp = TempDir::new().unwrap();
    let bin_dir = temp.child("bin");
    common::write_fake_git(bin_dir.path());
    let alfrc = temp.child("alfrc");
    let aliases = temp.child("aliases.txt");

    common::cmd()
        .current_dir(temp.path())
        .env("PATH", common::prepend_to_path(bin_dir.path()))
        .env("ALF_RC_FILE", alfrc.path())
        .env("ALF_ALIASES_FILE", aliases.path())
        .args(["connect", "https://github.com/DannyBen/alf-conf.git", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Connecting to https://github.com/DannyBen/alf-conf.git"))
        .stdout(predicate::str::contains("Storing location in"))
        .stdout(predicate::str::contains("Saving to"));
    let rc_content = std::fs::read_to_string(alfrc.path()).unwrap();
    // Migrated to ralf-conf
    assert!(rc_content.ends_with("/ralf-conf") || rc_content.ends_with("\\ralf-conf"));
}
