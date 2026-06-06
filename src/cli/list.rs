use crate::cli::{Exit, Result};

pub(crate) fn run() -> Result {
    let hosts = crate::list()?;
    if hosts.is_empty() {
        println!("(no hosts managed yet)");
    } else {
        for host in hosts {
            println!("{host}");
        }
    }
    Ok(Exit::Success)
}
