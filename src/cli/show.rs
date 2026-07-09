use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(ctx: &Context, host: &str) -> Result {
    print!("{}", ctx.show(host)?);
    Ok(Exit::Success)
}
