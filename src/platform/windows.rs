use std::sync::OnceLock;
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, HC_ACTION, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW,
    UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::errors::{GoofbindError, Result};
use crate::platform::PlatformUpdater;
use crate::structs::InternalMessage;

static HOOK_TX: OnceLock<Sender<InternalMessage>> = OnceLock::new();

unsafe extern "system" fn hook_callback(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode == HC_ACTION as i32 {
        // SAFETY: when ncode == HC_ACTION, lparam points to a valid KBDLLHOOKSTRUCT
        // supplied by the OS for the duration of this callback.
        let kbd_struct = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let msg = wparam.0 as u32;

        let is_pressed = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_released = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        if is_pressed || is_released {
            let mut vk = kbd_struct.vkCode;
            match vk {
                160 | 161 => vk = 16,
                162 | 163 => vk = 17,
                164 | 165 => vk = 18,
                92 => vk = 91,
                _ => {}
            }

            if let Some(tx) = HOOK_TX.get() {
                let _ = tx.send(InternalMessage::RawKey {
                    keycode: vk,
                    pressed: is_pressed,
                });
            }
        }
    }
    unsafe { CallNextHookEx(None, ncode, wparam, lparam) }
}

pub(crate) fn start_keybinds(
    tx: Sender<InternalMessage>,
    _app_id: Option<String>,
) -> Result<Option<PlatformUpdater>> {
    let tx_clone = tx.clone();
    let _ = HOOK_TX.set(tx);
    let (status_tx, status_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || unsafe {
        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_callback), None, 0) {
            Ok(hook) => {
                let _ = status_tx.send(Ok(()));
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}

                let _ = UnhookWindowsHookEx(hook);
                let _ = tx_clone.send(InternalMessage::FatalError(
                    "Windows hook message loop collapsed unexpectedly.".into(),
                ));
            }
            Err(_) => {
                let _ = status_tx.send(Err(GoofbindError::HookFailed));
            }
        }
    });

    status_rx.recv().unwrap_or(Err(GoofbindError::HookFailed))?;
    Ok(None)
}
