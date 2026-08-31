use crate::error::AppError;
use crate::ssh::inventory::{self, HostCandidate};
use crate::ssh::layout::Layout;

pub(crate) fn execute(layout: &Layout) -> Result<Vec<String>, AppError> {
    inventory::hosts(layout)?
        .into_iter()
        .map(|candidate| match candidate {
            HostCandidate::Managed(host) => Ok(host.host.to_string()),
            HostCandidate::Invalid { error, .. } => Err(error),
        })
        .collect()
}
