use crate::structs::KeybindInfo;

pub type PlatformUpdater = futures::channel::mpsc::UnboundedSender<Vec<KeybindInfo>>;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub(crate) use windows::start_keybinds;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::start_keybinds;
