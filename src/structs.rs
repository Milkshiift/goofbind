use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const VK_SHIFT: u32 = 16;
const VK_CONTROL: u32 = 17;
const VK_ALT: u32 = 18;
const VK_META: u32 = 91;

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize, Debug, Clone)]
pub struct KeybindInfo {
    pub id: String,
    pub name: Option<String>,
    pub keycode: Option<u32>,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub meta: bool,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum IncomingMessage {
    SetKeybinds { keybinds: Vec<KeybindInfo> },
}

#[derive(Serialize, Debug)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum OutgoingMessage {
    Pressed { id: String },
    Released { id: String },
    Error { message: String },
}

pub enum InternalMessage {
    Command(IncomingMessage),
    RawKey { keycode: u32, pressed: bool },
    WaylandEvent { id: String, pressed: bool },
    FatalError(String),
    Quit,
}

#[derive(Default)]
pub struct EngineState {
    pub keybinds: Vec<KeybindInfo>,
    pub pressed_keys: HashSet<u32>,
    pub active_keybinds: HashMap<String, u32>,
}

impl EngineState {
    pub fn handle_key<F>(&mut self, keycode: u32, is_pressed: bool, mut emit: F)
    where
        F: FnMut(OutgoingMessage),
    {
        if is_pressed {
            self.pressed_keys.insert(keycode);

            let ctrl = self.pressed_keys.contains(&VK_CONTROL) && keycode != VK_CONTROL;
            let alt = self.pressed_keys.contains(&VK_ALT) && keycode != VK_ALT;
            let shift = self.pressed_keys.contains(&VK_SHIFT) && keycode != VK_SHIFT;
            let meta = self.pressed_keys.contains(&VK_META) && keycode != VK_META;

            for kb in &self.keybinds {
                if let Some(kc) = kb.keycode
                    && kc == keycode
                    && kb.ctrl == ctrl
                    && kb.alt == alt
                    && kb.shift == shift
                    && kb.meta == meta
                    && let std::collections::hash_map::Entry::Vacant(entry) =
                        self.active_keybinds.entry(kb.id.clone())
                {
                    entry.insert(keycode);
                    emit(OutgoingMessage::Pressed { id: kb.id.clone() });
                }
            }
        } else {
            self.pressed_keys.remove(&keycode);

            self.active_keybinds.retain(|id, &mut stored_keycode| {
                if stored_keycode == keycode {
                    emit(OutgoingMessage::Released { id: id.clone() });
                    false
                } else {
                    true
                }
            });
        }
    }
}
