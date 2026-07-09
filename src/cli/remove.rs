use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(ctx: &Context, host: &str) -> Result {
    let status = ctx.remove(host)?;
    println!("{}", status.message(host));
    Ok(Exit::Success)
}
