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
use structs::{KeybindInfo, KeybindTrigger};

pub fn start_keybinds(tx: Sender<KeybindTrigger>, app_id: Option<String>) -> Result<()> {
    start_keybinds_internal(tx, app_id)
}

pub fn set_keybinds(keybinds: Vec<KeybindInfo>) -> Result<()> {
    set_keybinds_internal(keybinds)
}

pub fn get_current_shortcut() -> Result<String> {
    get_current_shortcut_internal()
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc::channel, thread};

    use crate::{
        set_keybinds, start_keybinds,
        structs::{KeybindInfo, KeybindTrigger},
    };
    #[test]
    fn demo() {
        let (tx, rx) = channel::<KeybindTrigger>();
        thread::spawn(|| {
            start_keybinds(tx, None).unwrap();
        });
        thread::sleep(std::time::Duration::from_secs(2));
        set_keybinds(vec![
            KeybindInfo {
                id: "1".to_owned(),
                name: Some("Does a thing!".to_owned()),
                shortcut: Some("shift+alt+m".to_owned()),
            },
            KeybindInfo {
                id: "2".to_owned(),
                name: Some("Does another thing!".to_owned()),
                shortcut: Some("shift+CTRL+a".to_owned()),
            },
        ])
        .unwrap();

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
