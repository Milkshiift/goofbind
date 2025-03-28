mod errors;
#[cfg(feature = "node")]
pub mod js;
mod structs;

#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod platform;

use std::sync::mpsc::Sender;

use errors::Result;
use platform::*;
use structs::{KeybindId, KeybindTrigger};

pub fn start_keybinds(tx: Sender<KeybindTrigger>) -> Result<()> {
    start_keybinds_internal(tx)
}

pub fn register_keybind(keybind: String, id: KeybindId) -> Result<()> {
    register_keybind_internal(keybind, id)
}
pub fn unregister_keybind(id: KeybindId) -> Result<()> {
    unregister_keybind_internal(id)
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc::channel, thread};

    use crate::{
        register_keybind, start_keybinds,
        structs::{KeybindTrigger, PreRegisterAction},
    };
    #[test]
    fn demo() {
        let (tx, rx) = channel::<KeybindTrigger>();
        thread::spawn(|| {
            start_keybinds(tx).unwrap();
        });
        thread::sleep(std::time::Duration::from_secs(2));
        #[cfg(target_os = "linux")]
        if crate::using_xdg() {
            crate::xdg_preregister_keybinds(vec![
                PreRegisterAction {
                    id: 1,
                    name: "Does a thing!".to_owned(),
                },
                PreRegisterAction {
                    id: 2,
                    name: "Does another thing!".to_owned(),
                },
            ])
            .unwrap();
        } else {
            register_keybind("shift+alt+m".to_string(), 1).unwrap();
            register_keybind("SHIFT+CTRL+a".to_string(), 2).unwrap();
        }
        #[cfg(not(target_os = "linux"))]
        {
            register_keybind("shift+alt+m".to_string(), 1).unwrap();
            register_keybind("SHIFT+CTRL+a".to_string(), 2).unwrap();
        }

        loop {
            match rx.recv() {
                Err(err) => {
                    panic!("{err}");
                }
                Ok(KeybindTrigger::Pressed(x)) => {
                    println!("pressed {}", x);
                }
                Ok(KeybindTrigger::Released(x)) => {
                    println!("released {}", x);
                }
            }
        }
    }
}
