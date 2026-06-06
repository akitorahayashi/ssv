use crate::cli::{Exit, Result};

pub(crate) fn run(host: &str) -> Result {
    let status = crate::remove(host)?;
    println!("{}", status.message(host));
    Ok(Exit::Success)
}
