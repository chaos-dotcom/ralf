use anyhow::Result;
use regex::Regex;
use std::fs;

pub fn run(code: String, subcode: Option<String>) -> Result<()> {
    let p = crate::paths::find_config_or_exit()?;
    let text = crate::config_merge::load_and_merge(&p)?;

    let re_top = Regex::new(&format!(r"^({}): *(.+)$", regex::escape(&code)))?;
    let re_sub = subcode
        .as_ref()
        .map(|sc| Regex::new(&format!(r"^( +)({}): *(.+)$", regex::escape(sc))))
        .transpose()?;

    let mut cmd1: Option<String> = None;

    for line in text.lines() {
        if cmd1.is_none() {
            if let Some(c) = re_top.captures(line) {
                cmd1 = Some(c.get(2).unwrap().as_str().to_string());
                if re_sub.is_none() {
                    println!("{}", cmd1.as_ref().unwrap());
                    return Ok(());
                }
            }
        } else if let Some(re) = &re_sub {
            if let Some(c) = re.captures(line) {
                let parent = cmd1.unwrap();
                let cmd2 = c.get(3).unwrap().as_str().to_string();
                if cmd2.starts_with('!') {
                    println!("{}", &cmd2[1..]);
                } else {
                    println!("{} {}", parent, cmd2);
                }
                return Ok(());
            }
        }
    }

    println!(
        "Error: No such alias: {}{}",
        code,
        subcode.map(|s| format!(" {}", s)).unwrap_or_default()
    );
    std::process::exit(1);
}
