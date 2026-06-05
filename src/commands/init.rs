use crate::error::AppError;
use crate::ssh_paths::SshPaths;

pub(crate) struct Init;

impl Init {
    pub(crate) fn execute(&self, paths: &SshPaths) -> Result<(), AppError> {
        paths.ensure_bootstrap()
    }
}
