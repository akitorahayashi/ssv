use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(ctx: &Context) -> Result {
    let hosts = ctx.list()?;
    if hosts.is_empty() {
        println!("(no hosts managed yet)");
    } else {
        for host in hosts {
            println!("{host}");
        }
    }
    Ok(Exit::Success)
}
