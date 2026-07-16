#[derive(Debug)]
pub enum GoofbindError {
    #[cfg(target_os = "windows")]
    HookFailed,
    #[cfg(target_os = "linux")]
    Portal(String),
}

impl std::fmt::Display for GoofbindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(target_os = "windows")]
            Self::HookFailed => write!(f, "Failed to install global OS keyboard hook."),
            #[cfg(target_os = "linux")]
            Self::Portal(err) => write!(f, "Wayland portal error: {err}"),
        }
    }
}

impl std::error::Error for GoofbindError {}

#[cfg(target_os = "linux")]
impl From<ashpd::Error> for GoofbindError {
    fn from(err: ashpd::Error) -> Self {
        Self::Portal(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GoofbindError>;
