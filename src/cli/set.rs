use crate::cli::{Exit, Result};

pub(crate) fn run(
    host: &str,
    hostname: Option<&str>,
    user: Option<&str>,
    port: Option<u16>,
) -> Result {
    let hostname = crate::set(host, hostname, user, port)?;
    println!("Updated '{host}' (HostName {hostname})");
    Ok(Exit::Success)
}
