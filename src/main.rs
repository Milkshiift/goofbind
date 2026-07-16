mod errors;
mod platform;
mod structs;

use std::env;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::channel;
use std::thread;

use structs::{EngineState, IncomingMessage, InternalMessage, OutgoingMessage};

fn emit_event(event: &OutgoingMessage) {
    let mut stdout = io::stdout().lock();
    if serde_json::to_writer(&mut stdout, event).is_ok() {
        let _ = stdout.write_all(b"\n");
        let _ = stdout.flush();
    }
}

fn main() {
    let app_id = env::args().nth(1);
    let (tx, rx) = channel::<InternalMessage>();

    let platform_updater = platform::start_keybinds(tx.clone(), app_id);

    let stdin_tx = tx;
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines().map_while(Result::ok) {
            match serde_json::from_str::<IncomingMessage>(&line) {
                Ok(msg) => {
                    let _ = stdin_tx.send(InternalMessage::Command(msg));
                }
                Err(e) => emit_event(&OutgoingMessage::Error {
                    message: format!("Invalid JSON payload: {e}"),
                }),
            }
        }
        // When Stdin EOF triggers (broken pipe), tell the process to die
        let _ = stdin_tx.send(InternalMessage::Quit);
    });

    let mut state = EngineState::default();

    while let Ok(msg) = rx.recv() {
        match msg {
            InternalMessage::Command(IncomingMessage::SetKeybinds { keybinds }) => {
                state.keybinds.clone_from(&keybinds);
                if let Some(updater) = &platform_updater {
                    let _ = updater.unbounded_send(keybinds);
                }
            }
            InternalMessage::RawKey { keycode, pressed } => {
                state.handle_key(keycode, pressed, |event| emit_event(&event));
            }
            InternalMessage::WaylandEvent { id, pressed } => {
                let event = if pressed {
                    OutgoingMessage::Pressed { id }
                } else {
                    OutgoingMessage::Released { id }
                };
                emit_event(&event);
            }
            InternalMessage::FatalError(e) => {
                emit_event(&OutgoingMessage::Error { message: e });
                std::process::exit(1);
            }
            InternalMessage::Quit => {
                break;
            }
        }
    }
}
