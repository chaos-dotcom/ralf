use assert_cmd::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
mod common;

#[test]
fn which_code_top_level() {
    let temp = TempDir::new().unwrap();
    common::copy_fixture_to(&temp, "generate/alf.conf", "alf.conf");

    common::cmd()
        .current_dir(temp.path())
        .args(["which", "g"])
        .assert()
        .success()
        .stdout("git\n");
}

#[test]
fn which_code_subcode() {
    let temp = TempDir::new().unwrap();
    common::copy_fixture_to(&temp, "generate/alf.conf", "alf.conf");

    common::cmd()
        .current_dir(temp.path())
        .args(["which", "g", "l"])
        .assert()
        .success()
        .stdout("git log --all --graph --date=relative\n");

    common::cmd()
        .current_dir(temp.path())
        .args(["which", "dc", "deploy"])
        .assert()
        .success()
        .stdout("docker stack deploy -c docker-compose.yml\n");

    common::cmd()
        .current_dir(temp.path())
        .args(["which", "dc", "upd"])
        .assert()
        .success()
        .stdout("docker-compose up -d\n");
}

#[test]
fn which_unknown_errors() {
    let temp = TempDir::new().unwrap();
    common::copy_fixture_to(&temp, "generate/alf.conf", "alf.conf");

    common::cmd()
        .current_dir(temp.path())
        .args(["which", "no"])
        .assert()
        .failure()
        .stdout(predicate::str::starts_with("Error: No such alias: no"));
}
