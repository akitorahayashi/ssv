use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(ctx: &Context, host: &str) -> Result {
    let new_url = ctx.link(host)?;
    println!("Linked repository to '{host}' (new remote URL: {new_url})");
    Ok(Exit::Success)
}
