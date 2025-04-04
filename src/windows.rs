use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::{mpsc::Sender, Mutex};
use std::sync::{LazyLock, OnceLock};

use uiohook_sys::{
    _event_type_EVENT_KEY_PRESSED, _event_type_EVENT_KEY_RELEASED, _uiohook_event, hook_run,
    hook_set_dispatch_proc, UIOHOOK_SUCCESS,
};

use crate::errors::{Result, VenbindError};
use crate::structs::{Shortcut, KeybindInfo, KeybindId, KeybindTrigger, Keybinds};

static KEYBINDS: LazyLock<Mutex<Keybinds>> = LazyLock::new(|| Mutex::new(Keybinds::default()));
static CURR_DOWN: LazyLock<Mutex<Option<(Shortcut, KeybindId)>>> =
    LazyLock::new(|| Mutex::new(None));
static TX: OnceLock<Sender<KeybindTrigger>> = OnceLock::new();

pub(crate) fn start_keybinds_internal(tx: Sender<KeybindTrigger>, _: Option<String>) -> Result<()> {
    TX.set(tx).unwrap();

    unsafe {
        hook_set_dispatch_proc(Some(dispatch_proc));
        if hook_run() != UIOHOOK_SUCCESS as i32 {
            return Err(VenbindError::LibUIOHookError);
        }
    };
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn dispatch_proc(event_ref: *mut _uiohook_event) {
    let event = *event_ref;
    if event.type_ == _event_type_EVENT_KEY_PRESSED {
        let keycode = uiohook_sys::platform::scancode_to_keycode(event.data.keyboard.keycode);

        const BUF_SIZE: usize = 8;
        let mut key_buffer: Vec<uiohook_sys::platform::wchar_t> = vec![0; BUF_SIZE];
        let str_count = uiohook_sys::platform::keycode_to_unicode(
            keycode,
            key_buffer.as_mut_ptr(),
            BUF_SIZE.try_into().unwrap(),
        );

        key_buffer.truncate(str_count.try_into().unwrap());
        let key = OsString::from_wide(&key_buffer);
        let shift = event.mask & uiohook_sys::MASK_SHIFT as u16 != 0;
        let alt = event.mask & uiohook_sys::MASK_ALT as u16 != 0;
        let ctrl = event.mask & uiohook_sys::MASK_CTRL as u16 != 0;
        let keybind = Shortcut {
            shift,
            alt,
            ctrl,
            character: if !key.is_empty() {
                Some(key.to_string_lossy().to_lowercase())
            } else {
                None
            },
        };
        let mut down = CURR_DOWN.lock().unwrap();
        if let Some((down_keybind, id)) = &*down {
            if *down_keybind == keybind {
                return; // prevent repeating Pressed triggers
            }
            TX.get()
                .unwrap()
                .send(KeybindTrigger::Released(id.clone()))
                .unwrap();
            down.take();
        }

        let keybinds = KEYBINDS.lock();
        if let Some(id) = keybinds.unwrap().get_keybind_id(&keybind) {
            TX.get()
                .unwrap()
                .send(KeybindTrigger::Pressed(id.clone()))
                .unwrap();
            down.replace((keybind, id));
        }
    } else if event.type_ == _event_type_EVENT_KEY_RELEASED {
        let mut down = CURR_DOWN.lock().unwrap();
        if let Some((_, id)) = &*down {
            TX.get()
                .unwrap()
                .send(KeybindTrigger::Released(id.clone()))
                .unwrap();
            down.take();
        }
    }
}

pub(crate) fn set_keybinds_internal(keybinds: Vec<KeybindInfo>) -> Result<()> {
    let mut keybinds_mutex = KEYBINDS.lock().unwrap();
    keybinds_mutex.clear();
    keybinds.iter().for_each(|x| {
        if x.shortcut.is_some() {
            keybinds_mutex.register_keybind(
                Shortcut::from_string(x.shortcut.clone().unwrap()),
                x.id.clone(),
            )
        }
    });
    Ok(())
}
