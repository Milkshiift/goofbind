use thiserror::Error;

pub type Result<T> = std::result::Result<T, VenbindError>;

#[derive(Debug, Error)]
pub enum VenbindError {
    #[error("Something went wrong with libuiohook")] // TODO: better log
    LibUIOHookError,
    #[cfg(all(target_os = "linux"))]
    #[error("ashpd error: {0}")]
    AshPdError(#[from] ashpd::Error),
}
