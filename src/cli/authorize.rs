use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(ctx: &Context, host: &str) -> Result {
    let target = ctx.authorize(host)?;
    println!("Authorized '{host}' public key on {target}");
    Ok(Exit::Success)
}
