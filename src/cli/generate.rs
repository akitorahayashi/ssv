use crate::cli::{Exit, Result};

pub(crate) fn run(host: &str, key_type: &str, user: Option<&str>, port: Option<u16>) -> Result {
    let public_key = crate::generate(host, key_type, user, port)?;
    println!("Generated SSH assets for '{host}'");
    print!("{public_key}");
    Ok(Exit::Success)
}
