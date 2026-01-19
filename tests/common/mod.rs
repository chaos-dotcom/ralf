#![allow(dead_code, unused_imports)]
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("ralf"))
}

pub fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bash-test")
        .join("fixtures")
}

pub fn copy_fixture_to(temp: &TempDir, rel: &str, dest_name: &str) -> PathBuf {
    let src = fixtures_root().join(rel);
    let dest = temp.path().join(dest_name);
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::copy(src, &dest).unwrap();
    dest
}

pub fn write_fake_git(bin_dir: &Path) {
    fs::create_dir_all(bin_dir).unwrap();
    let script = r#"#!/usr/bin/env sh
cmd="$1"; shift
case "$cmd" in
  clone)
    url="$1"; dest="$2"
    echo "Cloning into '$dest'..."
    mkdir -p "$dest"
    case "$url" in
      *alf-conf.git) touch "$dest/alf.conf" ;;
      *) touch "$dest/ralf.conf" ;;
    esac
    exit 0
    ;;
  pull)
    echo "Already up to date."
    exit 0
    ;;
  commit)
    echo "nothing to commit, working tree clean"
    exit 0
    ;;
  push)
    echo "pushed"
    exit 0
    ;;
  config)
    if [ "$1" = "--get" ] && [ "$2" = "remote.origin.url" ]; then
      echo "${FAKE_REMOTE_URL:-unset}"
      exit 0
    fi
    ;;
esac
exit 0
"#;
    let git_path = bin_dir.join("git");
    fs::write(&git_path, script).unwrap();
    let mut perms = fs::metadata(&git_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&git_path, perms).unwrap();
}

pub fn prepend_to_path(dir: &Path) -> String {
    let old = env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}
