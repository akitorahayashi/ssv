use crate::cli::{Exit, Result};

pub(crate) fn run(host: &str) -> Result {
    crate::remove(host)?;
    println!("Removed SSH assets for '{host}'");
    Ok(Exit::Success)
}
