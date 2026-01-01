use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn generate_completions(config_file: &Path) -> Result<String> {
    let text = fs::read_to_string(config_file)?;
    let ali1_re = Regex::new(r"^([a-z0-9\-]+):")?;
    let ali2_re = Regex::new(r"^ +([a-z0-9\-]+):")?;

    let mut out = String::new();
    out.push_str("# Completions\n");
    out.push_str("if command -v complete &> /dev/null ; then\n");

    let mut current_ali1: Option<String> = None;
    let mut comps: Vec<String> = Vec::new();

    for line in text.lines() {
        if let Some(c) = ali1_re.captures(line) {
            if let Some(a) = current_ali1.take() {
                if !comps.is_empty() {
                    out.push_str(&format!("  complete -W \"{}\" {}\n", comps.join(" "), a));
                }
                comps.clear();
            }
            current_ali1 = Some(c.get(1).unwrap().as_str().to_string());
        } else if let Some(c) = ali2_re.captures(line) {
            comps.push(c.get(1).unwrap().as_str().to_string());
        }
    }

    if let Some(a) = current_ali1 {
        if !comps.is_empty() {
            out.push_str(&format!("  complete -W \"{}\" {}\n", comps.join(" "), a));
        }
    }

    out.push_str("fi\n");
    Ok(out)
}
