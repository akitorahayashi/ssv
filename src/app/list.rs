use crate::context::Context;
use crate::error::AppError;
use crate::ssh::inventory::{self, HostCandidate};

pub(crate) fn execute(ctx: &Context) -> Result<Vec<String>, AppError> {
    inventory::hosts(ctx.layout())?
        .into_iter()
        .map(|candidate| match candidate {
            HostCandidate::Managed(host) => Ok(host.host.to_string()),
            HostCandidate::Invalid { error, .. } => Err(error),
        })
        .collect()
}
