mod wayland;
mod x11;

use std::env;
use std::sync::mpsc::Sender;

use crate::platform::PlatformUpdater;
use crate::structs::InternalMessage;

pub fn start_keybinds(
    tx: Sender<InternalMessage>,
    app_id: Option<String>,
) -> Option<PlatformUpdater> {
    if using_xdg() {
        Some(wayland::start_keybinds(tx, app_id))
    } else {
        x11::start_keybinds(tx)
    }
}

#[inline]
fn using_xdg() -> bool {
    env::var("XDG_SESSION_TYPE").is_ok_and(|x| x == "wayland")
        || env::var("WAYLAND_DISPLAY").is_ok()
        || env::var("GOOFBIND_USE_XDG_PORTAL").is_ok()
}
