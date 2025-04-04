use std::collections::HashMap;

pub type KeybindId = String;

#[cfg(feature = "node")]
use napi_derive::napi;

#[derive(Default)]
pub struct Keybinds {
    keybinds: HashMap<Shortcut, KeybindId>,
}

#[cfg_attr(feature = "node", napi(object))]
pub struct KeybindInfo {
    pub id: KeybindId,
    pub name: Option<String>,
    pub shortcut: Option<String>,
}

pub enum KeybindTrigger {
    Pressed(KeybindId),
    Released(KeybindId),
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct Shortcut {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub character: Option<String>,
}

impl Shortcut {
    pub fn from_string(keybind: String) -> Self {
        let lowercase_keybind = keybind.to_lowercase();
        let keys = lowercase_keybind.split("+");
        let mut shift = false;
        let mut alt = false;
        let mut ctrl = false;
        let mut character = None;
        keys.for_each(|x| match x {
            "shift" => shift = true,
            "alt" => alt = true,
            "ctrl" => ctrl = true,
            _ => character = Some(x.to_owned()),
        });
        Self {
            shift,
            alt,
            ctrl,
            character,
        }
    }
}

impl ToString for Shortcut {
    fn to_string(&self) -> String {
        let mut res = String::new();
        // formatted for https://specifications.freedesktop.org/shortcuts-spec/latest/#specification
        if self.shift {
            res.push_str("+SHIFT");
        }
        if self.alt {
            res.push_str("+ALT");
        }
        if self.ctrl {
            res.push_str("+CTRL");
        }
        if let Some(character) = &self.character {
            res.push_str(&format!("+{}", character));
        }
        res.trim_start_matches("+").to_owned()
    }
}

impl Keybinds {
    pub fn register_keybind(&mut self, keybind: Shortcut, id: KeybindId) {
        self.keybinds.insert(keybind, id);
    }
    pub fn clear(&mut self) {
        self.keybinds.clear();
    }
    pub fn get_keybind_id(&self, keybind: &Shortcut) -> Option<KeybindId> {
        self.keybinds.get(keybind).map(|x| x.clone())
    }
}
