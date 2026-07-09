use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(
    ctx: &Context,
    host: &str,
    hostname: Option<&str>,
    user: Option<&str>,
    port: Option<u16>,
) -> Result {
    let hostname = ctx.set(host, hostname, user, port)?;
    println!("Updated '{host}' (HostName {hostname})");
    Ok(Exit::Success)
}
