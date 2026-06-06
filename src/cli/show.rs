use crate::cli::{Exit, Result};

pub(crate) fn run(host: &str) -> Result {
    print!("{}", crate::show(host)?);
    Ok(Exit::Success)
}
