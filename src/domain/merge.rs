
use std::collections::HashMap;
use super::model::AliasBlock;

pub fn merge_blocks(mut base: Vec<AliasBlock>, overlay: Vec<AliasBlock>) -> Vec<AliasBlock> {
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

pub fn serialize_blocks(blocks: &[AliasBlock]) -> String {
    let mut out = String::new();
    for b in blocks {
        out.push_str(&format!("{}: {}\n", b.name, b.parent));
        for (sname, scmd) in &b.subs {
            out.push_str(&format!("  {}: {}\n", sname, scmd));
        }
    }
    out
}
