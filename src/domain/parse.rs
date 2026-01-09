use anyhow::Result;
use regex::Regex;

use super::model::AliasBlock;

pub fn parse_text(text: &str) -> Result<Vec<AliasBlock>> {
    let re = Regex::new(r"^( *)([A-Za-z0-9\-]+): *(.+)$")?;
    let mut blocks: Vec<AliasBlock> = Vec::new();
    let mut current: Option<AliasBlock> = None;

    for line in text.lines() {
        if let Some(c) = re.captures(line) {
            let indent = c.get(1).unwrap().as_str();
            let name = c.get(2).unwrap().as_str().to_string();
            let cmd = c.get(3).unwrap().as_str().to_string();

            if indent.is_empty() {
                if let Some(b) = current.take() {
                    blocks.push(b);
                }
                current = Some(AliasBlock {
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
