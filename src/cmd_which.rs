use anyhow::Result;

pub fn run(code: String, subcode: Option<String>) -> Result<()> {
    let p = crate::paths::find_config_or_exit()?;
    let blocks = crate::config_merge::load_and_merge_model(&p)?;

    if let Some(b) = blocks.iter().find(|b| b.name == code) {
        if let Some(sc) = subcode.as_ref() {
            if let Some((_, scmd)) = b.subs.iter().find(|(n, _)| n == sc) {
                if scmd.starts_with('!') {
                    println!("{}", &scmd[1..]);
                } else {
                    println!("{} {}", b.parent, scmd);
                }
                return Ok(());
            }
        } else {
            println!("{}", b.parent);
            return Ok(());
        }
    }

    println!(
        "Error: No such alias: {}{}",
        code,
        subcode.map(|s| format!(" {}", s)).unwrap_or_default()
    );
    std::process::exit(1);
}
