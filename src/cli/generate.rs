use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(
    ctx: &Context,
    host: &str,
    hostname: Option<&str>,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result {
    let public_key = ctx.generate(host, hostname, key_type, user, port)?;
    println!("Generated SSH assets for '{host}'");
    print!("{public_key}");
    Ok(Exit::Success)
}
