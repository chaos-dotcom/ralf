use crate::domain::{merge_blocks, parse_text, serialize_blocks, AliasBlock};
use crate::paths;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn resolve_machine_id(p: &paths::Paths) -> String {
    // 1) Repo marker wins
    let marker = p.repo_path.join(".ralf_machine");
    if marker.exists() {
        if let Ok(s) = std::fs::read_to_string(&marker) {
            let t = s.trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }
    // Legacy marker migration: .alf_machine -> .ralf_machine
    let legacy = p.repo_path.join(".alf_machine");
    if legacy.exists() {
        if let Ok(s) = std::fs::read_to_string(&legacy) {
            let t = s.trim().to_string();
            if !t.is_empty() {
                let _ = std::fs::rename(&legacy, p.repo_path.join(".ralf_machine"));
                return t;
            }
        }
    }

    // 2) Environment variables (new + legacy, common casings)
    if let Ok(s) = std::env::var("RALF_MACHINE")
        .or_else(|_| std::env::var("ALF_MACHINE"))
        .or_else(|_| std::env::var("ralf_MACHINE"))
        .or_else(|_| std::env::var("alf_MACHINE"))
    {
        let t = s.trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }

    // 3) Host fallback
    if let Ok(h) = std::env::var("HOSTNAME") {
        let t = h.trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    if let Ok(out) = Command::new("hostname").arg("-s").output() {
        let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    "unknown".to_string()
}

pub fn overlay_paths(p: &paths::Paths, machine: &str) -> (PathBuf, PathBuf) {
    (
        p.repo_path
            .join("machines")
            .join(format!("{}.conf", machine)),
        p.repo_path.join("ralf.local.conf"),
    )
}

pub fn load_and_merge(p: &paths::Paths) -> Result<String> {
    let base_text = fs::read_to_string(&p.config_file)?;
    let mut blocks = parse_text(&base_text)?;
    let machine = resolve_machine_id(p);
    let (machine_path, local_path) = overlay_paths(p, &machine);

    if machine_path.exists() {
        let overlay = fs::read_to_string(&machine_path)?;
        let ob = parse_text(&overlay)?;
        blocks = merge_blocks(blocks, ob);
    }
    if local_path.exists() {
        let overlay = fs::read_to_string(&local_path)?;
        let ob = parse_text(&overlay)?;
        blocks = merge_blocks(blocks, ob);
    }
    // Legacy local overlay fallback: alf.local.conf
    if !local_path.exists() {
        let legacy_local = p.repo_path.join("alf.local.conf");
        if legacy_local.exists() {
            let overlay = fs::read_to_string(&legacy_local)?;
            let ob = parse_text(&overlay)?;
            blocks = merge_blocks(blocks, ob);
        }
    }
    Ok(serialize_blocks(&blocks))
}

pub fn load_and_merge_model(p: &paths::Paths) -> Result<Vec<AliasBlock>> {
    let base_text = std::fs::read_to_string(&p.config_file)?;
    let mut blocks = parse_text(&base_text)?;
    let machine = resolve_machine_id(p);
    let (machine_path, local_path) = overlay_paths(p, &machine);

    if machine_path.exists() {
        let overlay = std::fs::read_to_string(&machine_path)?;
        let ob = parse_text(&overlay)?;
        blocks = merge_blocks(blocks, ob);
    }
    if local_path.exists() {
        let overlay = std::fs::read_to_string(&local_path)?;
        let ob = parse_text(&overlay)?;
        blocks = merge_blocks(blocks, ob);
    }
    // Legacy local overlay fallback: alf.local.conf
    if !local_path.exists() {
        let legacy_local = p.repo_path.join("alf.local.conf");
        if legacy_local.exists() {
            let overlay = std::fs::read_to_string(&legacy_local)?;
            let ob = parse_text(&overlay)?;
            blocks = merge_blocks(blocks, ob);
        }
    }
    Ok(blocks)
}
