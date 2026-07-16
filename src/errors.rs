use thiserror::Error;

use crate::structs::KeybindTrigger;

pub type Result<T> = std::result::Result<T, GoofbindError>;

#[derive(Debug, Error)]
pub enum GoofbindError {
    #[error("Something went wrong with libuiohook")] // TODO: better log
    LibUIOHookError,
    #[error("No communication with main thread.")]
    MpscSendError(#[from] std::sync::mpsc::SendError<KeybindTrigger>),

    #[cfg(target_os = "linux")]
    #[error("Can't use on XDG!")]
    UnsupportedOnXdg,
    #[cfg(target_os = "linux")]
    #[error("ashpd error: {0}")]
    AshPdError(#[from] ashpd::Error),
}
