use std::collections::HashSet;

pub type KeybindId = String;

#[cfg(feature = "node")]
use napi_derive::napi;

#[derive(Default)]
pub struct Keybinds {
    keybinds: Vec<(Shortcut, KeybindId)>,
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct Shortcut {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub keys: HashSet<String>,
}

impl Shortcut {
    pub fn from_string(keybind: String) -> Self {
        let lowercase_keybind = keybind.to_lowercase();
        let keys = lowercase_keybind.split("+");
        let mut shift = false;
        let mut alt = false;
        let mut ctrl = false;
        let mut meta = false;
        let mut chars = HashSet::new();
        keys.for_each(|x| match x {
            "shift" => shift = true,
            "alt" => alt = true,
            "ctrl" => ctrl = true,
            "meta" => meta = true,
            _ => {
                chars.insert(x.to_owned());
            }
        });
        Self {
            shift,
            alt,
            ctrl,
            meta,
            keys: chars,
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
        if self.meta {
            res.push_str("+META");
        }
        if !self.keys.is_empty() {
            res.push_str(
                &self
                    .keys
                    .iter()
                    .map(|x| format!("+{}", x))
                    .collect::<String>(),
            );
        }
        res.trim_start_matches("+").to_owned()
    }
}

impl Keybinds {
    pub fn register_keybind(&mut self, keybind: Shortcut, id: KeybindId) {
        self.keybinds.push((keybind, id));
    }
    pub fn clear(&mut self) {
        self.keybinds.clear();
    }
    pub fn get_active_keybinds(&self, keys: &Shortcut) -> Vec<KeybindId> {
        self.keybinds
            .iter()
            .filter(|x| {
                x.0.keys.is_subset(&keys.keys)
                    && (!x.0.alt || (x.0.alt == keys.alt))
                    && (!x.0.ctrl || (x.0.ctrl == keys.ctrl))
                    && (!x.0.shift || (x.0.shift == keys.shift))
                    && (!x.0.meta || (x.0.meta == keys.meta))
            })
            .map(|x| x.1.clone())
            .collect()
    }
}
