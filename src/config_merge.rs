use crate::paths;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
struct Block {
    name: String,
    parent: String,
    subs: Vec<(String, String)>,
}

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
    let mut blocks = parse(&base_text)?;
    let machine = resolve_machine_id(p);
    let (machine_path, local_path) = overlay_paths(p, &machine);

    if machine_path.exists() {
        let overlay = fs::read_to_string(&machine_path)?;
        let ob = parse(&overlay)?;
        blocks = merge(blocks, ob);
    }
    if local_path.exists() {
        let overlay = fs::read_to_string(&local_path)?;
        let ob = parse(&overlay)?;
        blocks = merge(blocks, ob);
    }
    // Legacy local overlay fallback: alf.local.conf
    if !local_path.exists() {
        let legacy_local = p.repo_path.join("alf.local.conf");
        if legacy_local.exists() {
            let overlay = fs::read_to_string(&legacy_local)?;
            let ob = parse(&overlay)?;
            blocks = merge(blocks, ob);
        }
    }
    Ok(serialize(&blocks))
}

fn parse(text: &str) -> Result<Vec<Block>> {
    let re = Regex::new(r"^( *)([A-Za-z0-9\-]+): *(.+)$")?;
    let mut blocks: Vec<Block> = Vec::new();
    let mut current: Option<Block> = None;

    for line in text.lines() {
        if let Some(c) = re.captures(line) {
            let indent = c.get(1).unwrap().as_str();
            let name = c.get(2).unwrap().as_str().to_string();
            let cmd = c.get(3).unwrap().as_str().to_string();

            if indent.is_empty() {
                if let Some(b) = current.take() {
                    blocks.push(b);
                }
                current = Some(Block {
                    name,
                    parent: cmd,
                    subs: Vec::new(),
                });
            } else {
                if let Some(ref mut b) = current {
                    b.subs.push((name, cmd));
                }
            }
        }
    }
    if let Some(b) = current.take() {
        blocks.push(b);
    }
    Ok(blocks)
}

fn merge(mut base: Vec<Block>, overlay: Vec<Block>) -> Vec<Block> {
    // Index base top-level aliases for quick lookup.
    let mut index: HashMap<String, usize> = HashMap::new();
    for (i, b) in base.iter().enumerate() {
        index.insert(b.name.clone(), i);
    }

    for ob in overlay {
        if let Some(&i) = index.get(&ob.name) {
            // Replace parent and merge subs
            base[i].parent = ob.parent.clone();

            // Map existing subs to preserve positions; replace or append new.
            let mut sub_idx: HashMap<String, usize> = HashMap::new();
            for (j, (sname, _)) in base[i].subs.iter().enumerate() {
                sub_idx.insert(sname.clone(), j);
            }
            for (sname, scmd) in ob.subs.iter() {
                if let Some(&j) = sub_idx.get(sname) {
                    base[i].subs[j].1 = scmd.clone();
                } else {
                    base[i].subs.push((sname.clone(), scmd.clone()));
                }
            }
        } else {
            // New alias, append at the end, keep overlay order
            index.insert(ob.name.clone(), base.len());
            base.push(ob);
        }
    }
    base
}

fn serialize(blocks: &Vec<Block>) -> String {
    let mut out = String::new();
    for b in blocks {
        out.push_str(&format!("{}: {}\n", b.name, b.parent));
        for (sname, scmd) in &b.subs {
            out.push_str(&format!("  {}: {}\n", sname, scmd));
        }
    }
    out
}
