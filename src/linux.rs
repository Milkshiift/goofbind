use ashpd::{
    desktop::{global_shortcuts::*, *},
    register_host_app,
    zbus::export::futures_util::StreamExt,
    AppID,
};
use futures::{executor::block_on, future::Either};
use std::{
    cell::RefCell,
    env,
    str::FromStr,
    sync::{mpsc::Sender, LazyLock, Mutex, OnceLock},
};
use uiohook_sys::{
    _event_type_EVENT_KEY_PRESSED, _event_type_EVENT_KEY_RELEASED, _uiohook_event, hook_run,
    hook_set_dispatch_proc, UIOHOOK_SUCCESS,
};
use xcb::Extension;
use xkbcommon::xkb::{self, State};

use crate::structs::{Keybind, KeybindId, KeybindTrigger, Keybinds};
use crate::{
    errors::{Result, VenbindError},
    js::PreRegisterAction,
};

static KEYBINDS: LazyLock<Mutex<Keybinds>> = LazyLock::new(|| Mutex::new(Keybinds::default()));
static CURR_DOWN: LazyLock<Mutex<Option<(Keybind, KeybindId)>>> =
    LazyLock::new(|| Mutex::new(None));
static TX: OnceLock<Sender<KeybindTrigger>> = OnceLock::new();

static XDG_STATE: LazyLock<Mutex<Option<XDGState>>> = LazyLock::new(|| Mutex::new(None));

thread_local! {
    static XKBCOMMON_STATE: RefCell<Option<State>> = RefCell::new(None);
}

struct XDGState<'a> {
    portal: global_shortcuts::GlobalShortcuts<'a>,
    session: Session<'a, ashpd::desktop::global_shortcuts::GlobalShortcuts<'a>>,
}

pub(crate) fn start_keybinds_internal(
    tx: Sender<KeybindTrigger>,
    app_id: Option<String>,
) -> Result<()> {
    TX.set(tx).unwrap();
    if using_xdg() {
        block_on(xdg_start_keybinds(app_id))
    } else {
        uiohook_start_keybinds()
    }
}

pub(crate) fn register_keybind_internal(keybind: String, id: KeybindId) -> Result<()> {
    if using_xdg() {
        return Err(VenbindError::UnsupportedOnXdg);
    }
    uiohook_register_keybind(keybind, id)
}

pub(crate) fn unregister_keybind_internal(id: KeybindId) -> Result<()> {
    let mut keybinds = KEYBINDS.lock().unwrap();
    keybinds.unregister_keybind(id);
    Ok(())
}

async fn xdg_start_keybinds(app_id: Option<String>) -> Result<()> {
    if let Some(app_id) = app_id {
        if let Err(err) = register_host_app(AppID::from_str(&app_id).unwrap()).await {
            eprintln!("Couldn't use registry (chances are your version of xdg-desktop-portal is old): {err}")
        }
    }
    let mut state = XDG_STATE.lock().unwrap();
    let portal = GlobalShortcuts::new().await?;
    let session = portal.create_session().await?;

    state.replace(XDGState { portal, session });
    drop(state);

    xdg_input_thread().await?;

    Ok(())
}

async fn xdg_input_thread() -> Result<()> {
    let (mut activated, mut deactivted) = {
        let state = XDG_STATE.lock().unwrap();
        if let Some(state) = state.as_ref() {
            let activated = state.portal.receive_activated().await?;
            let deactivated = state.portal.receive_deactivated().await?;
            (activated, deactivated)
        } else {
            panic!("This Thread should not be active no XDG state");
        }
    };
    loop {
        match futures::future::select(activated.next(), deactivted.next()).await {
            Either::Left((Some(activated), _)) => TX
                .get()
                .unwrap()
                .send(KeybindTrigger::Pressed(activated.shortcut_id().to_owned()))?,
            Either::Right((Some(deactivated), _)) => TX.get().unwrap().send(
                KeybindTrigger::Released(deactivated.shortcut_id().to_owned()),
            )?,
            _ => {
                eprintln!("Unexpected output from GlobalShortcuts!");
            }
        }
    }
}

pub(crate) fn xdg_preregister_keybinds(actions: Vec<PreRegisterAction>) -> Result<()> {
    if !using_xdg() {
        return Err(VenbindError::UnsupportedOnXdg);
    }
    let shortcuts: Vec<NewShortcut> = actions
        .iter()
        .map(|x| NewShortcut::new(&x.id, &x.name))
        .collect();
    let lock = XDG_STATE.lock().unwrap();
    if let Some(state) = lock.as_ref() {
        let listshortcuts = block_on(state.portal.list_shortcuts(&state.session))?.response()?;
        let curr_shortcuts = listshortcuts.shortcuts();

        if !actions
            .iter()
            .all(|x| curr_shortcuts.iter().any(|y| y.id() == x.id))
        {
            block_on(
                state
                    .portal
                    .bind_shortcuts(&state.session, &shortcuts, None),
            )?;
        }
    } else {
        eprintln!("No GlobalShortcuts state was found! skipping preregistery.");
    }
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn uiohook_dispatch_proc(event_ref: *mut _uiohook_event) {
    let event = &*event_ref;
    if event.type_ == _event_type_EVENT_KEY_PRESSED {
        XKBCOMMON_STATE.with(|state| {
            let state_borrow = state.borrow();
            let state = state_borrow.as_ref().unwrap();
            let keycode = uiohook_sys::platform::scancode_to_keycode(event.data.keyboard.keycode);
            let key = state.key_get_utf8(keycode.into());
            let shift = event.mask & uiohook_sys::MASK_SHIFT as u16 != 0;
            let alt = event.mask & uiohook_sys::MASK_ALT as u16 != 0;
            let ctrl = event.mask & uiohook_sys::MASK_CTRL as u16 != 0;
            let keybind = Keybind {
                shift,
                alt,
                ctrl,
                character: if !key.is_empty() { Some(key) } else { None },
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
        });
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

fn uiohook_start_keybinds() -> Result<()> {
    let (connection, _screen) =
        xcb::Connection::connect_with_extensions(None, &[Extension::Xkb], &[]).unwrap();
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    xkb::x11::setup_xkb_extension(
        &connection,
        xkb::x11::MIN_MAJOR_XKB_VERSION,
        xkb::x11::MIN_MINOR_XKB_VERSION,
        xkb::x11::SetupXkbExtensionFlags::NoFlags,
        &mut 0,
        &mut 0,
        &mut 0,
        &mut 0,
    );
    let device_id = xkb::x11::get_core_keyboard_device_id(&connection);
    let keymap = xkb::x11::keymap_new_from_device(
        &context,
        &connection,
        device_id,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    );
    drop(connection);
    // don't make a state with an xcb connection (state_new_from_device) so it only chooses the first layout
    // TODO: if someone's first selected layout is not a latin based layout horrible things happen
    let state = xkb::State::new(&keymap);
    XKBCOMMON_STATE.replace(Some(state));
    unsafe {
        hook_set_dispatch_proc(Some(uiohook_dispatch_proc));
        if hook_run() != UIOHOOK_SUCCESS as i32 {
            return Err(VenbindError::LibUIOHookError);
        }
    };
    Ok(())
}

fn uiohook_register_keybind(keybind: String, id: KeybindId) -> Result<()> {
    let mut keybinds = KEYBINDS.lock().unwrap();
    keybinds.register_keybind(Keybind::from_string(keybind.clone()), id);
    Ok(())
}

#[inline]
pub(crate) fn using_xdg() -> bool {
    env::var("XDG_SESSION_TYPE").is_ok_and(|x| x == "wayland".to_owned())
        || env::var("WAYLAND_DISPLAY").is_ok()
        || env::var("VENBIND_USE_XDG_PORTAL").is_ok()
}
