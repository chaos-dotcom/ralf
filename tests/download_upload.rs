use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;
mod common;

fn setup_repo_with_rc(temp: &TempDir) -> (String, String) {
    let repo = temp.child("ralf-conf");
    repo.create_dir_all().unwrap();
    fs::write(repo.child("ralf.conf"), b"g: git\n").unwrap();
    let rc = temp.child("alfrc");
    fs::write(&rc, repo.path().to_string_lossy().as_bytes()).unwrap();
    (rc.path().to_string_lossy().into_owned(), repo.path().to_string_lossy().into_owned())
}

#[test]
fn download_works() {
    let temp = TempDir::new().unwrap();
    let bin_dir = temp.child("bin");
    common::write_fake_git(bin_dir.path());
    let (rc, _repo) = setup_repo_with_rc(&temp);
    let aliases = temp.child("aliases.txt");

    common::cmd()
        .current_dir(temp.path())
        .env("PATH", common::prepend_to_path(bin_dir.path()))
        .env("ALF_RC_FILE", rc)
        .env("ALF_ALIASES_FILE", aliases.path())
        .arg("download")
        .assert()
        .success()
        .stdout(predicate::str::contains("Pulling from repository to"))
        .stdout(predicate::str::contains("Saving to"));
}

#[test]
fn upload_works() {
    let temp = TempDir::new().unwrap();
    let bin_dir = temp.child("bin");
    common::write_fake_git(bin_dir.path());
    let (rc, _repo) = setup_repo_with_rc(&temp);

    common::cmd()
        .current_dir(temp.path())
        .env("PATH", common::prepend_to_path(bin_dir.path()))
        .env("ALF_RC_FILE", rc)
        .arg("upload")
        .assert()
        .success()
        .stdout(predicate::str::contains("Pushing"));
}
