use crate::cli::{Exit, Result};

pub(crate) fn run() -> Result {
    let status = crate::init()?;
    println!("{status}");
    Ok(Exit::Success)
}
