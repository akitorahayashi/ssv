use crate::cli::{Exit, Result};

pub(crate) fn run() -> Result {
    crate::init()?;
    println!("SSH bootstrap is ready");
    Ok(Exit::Success)
}
