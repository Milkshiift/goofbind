use std::collections::HashSet;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::{mpsc::Sender, Mutex};
use std::sync::{LazyLock, OnceLock};

use uiohook_sys::{
    _event_type_EVENT_KEY_PRESSED, _event_type_EVENT_KEY_RELEASED, _uiohook_event, hook_run,
    hook_set_dispatch_proc, UIOHOOK_SUCCESS,
};

use crate::errors::{Result, VenbindError};
use crate::structs::{KeybindId, KeybindInfo, KeybindTrigger, Keybinds, Shortcut};

static KEYBINDS: LazyLock<Mutex<Keybinds>> = LazyLock::new(|| Mutex::new(Keybinds::default()));
static CURR_DOWN: LazyLock<Mutex<Shortcut>> = LazyLock::new(|| {
    Mutex::new(Shortcut {
        shift: false,
        alt: false,
        ctrl: false,
        meta: false,
        keys: HashSet::new(),
    })
});
static CURR_ACTIVE_KEYBINDS: LazyLock<Mutex<HashSet<KeybindId>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
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
pub extern "C" fn dispatch_proc(event_ref: *mut _uiohook_event) {
    let event = unsafe { *event_ref };
    if event.type_ == _event_type_EVENT_KEY_PRESSED || event.type_ == _event_type_EVENT_KEY_RELEASED
    {
        let keycode =
            unsafe { uiohook_sys::platform::scancode_to_keycode(event.data.keyboard.keycode) };

        const BUF_SIZE: usize = 8;
        let mut key_buffer: Vec<uiohook_sys::platform::wchar_t> = vec![0; BUF_SIZE];
        let str_count = unsafe {
            uiohook_sys::platform::keycode_to_unicode(
                keycode,
                key_buffer.as_mut_ptr(),
                BUF_SIZE.try_into().unwrap(),
            )
        };

        key_buffer.truncate(str_count.try_into().unwrap());
        let key = OsString::from_wide(&key_buffer);
        let shift = event.mask & uiohook_sys::MASK_SHIFT as u16 != 0;
        let alt = event.mask & uiohook_sys::MASK_ALT as u16 != 0;
        let ctrl = event.mask & uiohook_sys::MASK_CTRL as u16 != 0;
        let meta = event.mask & uiohook_sys::MASK_META as u16 != 0;

        let mut curr_down = CURR_DOWN.lock().unwrap();
        curr_down.alt = alt;
        curr_down.shift = shift;
        curr_down.ctrl = ctrl;
        curr_down.meta = meta;
        if !key.is_empty() {
            if event.type_ == _event_type_EVENT_KEY_PRESSED {
                curr_down.keys.insert(key.to_string_lossy().to_lowercase());
            } else {
                curr_down.keys.remove(&key.to_string_lossy().to_lowercase());
            }
        }
        let keybinds = KEYBINDS.lock().unwrap();
        let active: HashSet<String> = keybinds
            .get_active_keybinds(&curr_down)
            .into_iter()
            .collect();
        let mut curr_active_keybinds = CURR_ACTIVE_KEYBINDS.lock().unwrap();
        let pressed_keybinds = active.difference(&curr_active_keybinds);
        let released_keybinds = curr_active_keybinds.difference(&active);
        for pressed in pressed_keybinds {
            TX.get()
                .unwrap()
                .send(KeybindTrigger::Pressed(pressed.clone()))
                .unwrap();
        }
        for released in released_keybinds {
            TX.get()
                .unwrap()
                .send(KeybindTrigger::Released(released.clone()))
                .unwrap();
        }
        curr_active_keybinds.clear();
        curr_active_keybinds.extend(active);
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

pub(crate) fn get_current_shortcut_internal() -> Result<String> {
    let down = CURR_DOWN.lock().unwrap();
    Ok(down.to_string())
}
