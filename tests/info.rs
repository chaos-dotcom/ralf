use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;
mod common;

#[test]
fn info_when_files_exist() {
    let temp = TempDir::new().unwrap();
    let bin_dir = temp.child("bin");
    common::write_fake_git(bin_dir.path());

    let repo = temp.child("alf-conf");
    repo.create_dir_all().unwrap();
    // Legacy alf.conf to exercise migration
    fs::write(repo.child("alf.conf"), b"g: git\n").unwrap();
    fs::create_dir_all(repo.child(".git")).unwrap();

    let rc = temp.child("alfrc");
    fs::write(&rc, repo.path().to_string_lossy().as_bytes()).unwrap();
    let aliases = temp.child("aliases.txt");
    fs::write(&aliases, b"# dummy").unwrap();

    common::cmd()
        .current_dir(temp.path())
        .env("PATH", common::prepend_to_path(bin_dir.path()))
        .env(
            "FAKE_REMOTE_URL",
            "https://github.com/DannyBen/ralf-conf.git",
        )
        .env("ALF_RC_FILE", rc.path())
        .env("ALF_ALIASES_FILE", aliases.path())
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("Environment:"))
        .stdout(predicate::str::contains("ALF_RC_FILE:"))
        .stdout(predicate::str::contains("ALF_ALIASES_FILE:"))
        .stdout(predicate::str::contains("Paths:"))
        .stdout(predicate::str::contains("GitHub:"))
        .stdout(predicate::str::contains(
            "remote:            https://github.com/DannyBen/ralf-conf.git",
        ));
}
