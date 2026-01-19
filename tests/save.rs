use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
mod common;

#[test]
fn save_writes_aliases_and_tip() {
    let temp = TempDir::new().unwrap();
    common::copy_fixture_to(&temp, "generate/alf.conf", "alf.conf");
    let aliases = temp.child("aliases.txt");
    let mut c = common::cmd();
    c.current_dir(temp.path())
        .env("ALF_ALIASES_FILE", aliases.path())
        .arg("save");
    c.assert()
        .success()
        .stdout(predicate::str::contains("Saving to"))
        .stdout(predicate::str::contains(
            "To apply the new aliases to the current session, run:",
        ))
        .stdout(predicate::str::contains("$ source"));
    aliases.assert(predicates::path::exists());
}

#[test]
fn save_missing_config_errors() {
    let temp = TempDir::new().unwrap();
    let aliases = temp.child("aliases.txt");
    let mut c = common::cmd();
    c.current_dir(temp.path())
        .env("ALF_ALIASES_FILE", aliases.path())
        .arg("save");
    c.assert().failure().stdout(predicate::str::starts_with(
        "ERROR: Cannot find config file",
    ));
    aliases.assert(predicates::path::missing());
}
