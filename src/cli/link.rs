use crate::cli::{Exit, Result};

pub(crate) fn run(host: &str) -> Result {
    let new_url = crate::link(host)?;
    println!("Linked repository to '{host}' (new remote URL: {new_url})");
    Ok(Exit::Success)
}
