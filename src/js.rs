use std::{
    sync::{mpsc::channel, LazyLock, Mutex},
    thread,
};

use napi::{
    bindgen_prelude::*,
    threadsafe_function::{
        ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
    },
};
use napi_derive::napi;

use crate::structs::{KeybindInfo, KeybindTrigger};


static JS_ERROR_HANDLE: LazyLock<Mutex<Option<ThreadsafeFunction<String, ErrorStrategy::Fatal>>>> =
    LazyLock::new(|| Mutex::new(None));

macro_rules! pass_to_js_error_handle {
    ($func:expr) => {
        let _ = $func.inspect_err(|e| {
            if let Some(err_func) = &*JS_ERROR_HANDLE.lock().unwrap() {
                err_func.call(format!("{e}"), ThreadsafeFunctionCallMode::Blocking);
            }
        });
    };
}

#[napi(ts_args_type = "callback: (id: string, keyup: boolean) => void, app_id: string | null")]
pub fn start_keybinds(callback: JsFunction, app_id: Option<String>) -> Result<()> {
    let (tx, rx) = channel::<KeybindTrigger>();
    thread::spawn(|| {
        pass_to_js_error_handle!(crate::start_keybinds(tx, app_id));
    });
    let thread_function: ThreadsafeFunction<(String, bool), ErrorStrategy::Fatal> = callback
        .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<(String, bool)>| {
            ctx.env.create_string_from_std(ctx.value.0).and_then(|y| {
                ctx.env
                    .get_boolean(ctx.value.1)
                    .and_then(|x| (y, x).into_vec(ctx.env.raw()))
            })
        })?;
    thread::spawn(move || loop {
        match rx.recv() {
            Err(err) => {
                panic!("{err}");
            }
            Ok(KeybindTrigger::Pressed(x)) => {
                thread_function.call((x, false), ThreadsafeFunctionCallMode::Blocking);
            }
            Ok(KeybindTrigger::Released(x)) => {
                thread_function.call((x, true), ThreadsafeFunctionCallMode::Blocking);
            }
        }
    });

    Ok(())
}

#[napi]
pub fn set_keybinds(
    #[napi(ts_arg_type = "KeybindInfo[]")] keybinds: Vec<KeybindInfo>,
) {
    pass_to_js_error_handle!(crate::set_keybinds(keybinds));
}

#[napi(ts_args_type = "callback: (error: string) => void")]
pub fn define_error_handle(callback: JsFunction) -> Result<()> {
    let error_function: ThreadsafeFunction<String, ErrorStrategy::Fatal> = callback
        .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<String>| {
            ctx.env.create_string_from_std(ctx.value).map(|v| vec![v])
        })?;
    JS_ERROR_HANDLE.lock().unwrap().replace(error_function);
    Ok(())
}
