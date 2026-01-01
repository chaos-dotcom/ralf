use anyhow::Result;

pub fn run() -> Result<()> {
    let s = crate::generator::generate_config()?;
    print!("{s}");
    Ok(())
}
