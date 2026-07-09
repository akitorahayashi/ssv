use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(ctx: &Context) -> Result {
    let status = ctx.init()?;
    println!("{status}");
    Ok(Exit::Success)
}
