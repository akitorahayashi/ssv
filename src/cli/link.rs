use crate::cli::{Exit, Result};
use crate::context::Context;
use crate::error::IoResultExt;
use std::path::Path;

pub(crate) fn run(ctx: &Context, host: &str) -> Result {
    let repository_start = std::env::current_dir().path_ctx(Path::new("."))?;
    let new_url = ctx.link(&repository_start, host)?;
    println!("Linked repository to '{host}' (new remote URL: {new_url})");
    Ok(Exit::Success)
}
