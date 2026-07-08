use crate::cli::{Exit, Result};

pub(crate) fn run(host: &str) -> Result {
    let target = crate::authorize(host)?;
    println!("Authorized '{host}' public key on {target}");
    Ok(Exit::Success)
}
