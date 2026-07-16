use std::str::FromStr;
use std::sync::mpsc::Sender;

use ashpd::{
    AppID,
    desktop::global_shortcuts::{GlobalShortcuts, NewShortcut},
    register_host_app,
};
use futures::StreamExt;
use futures::executor::block_on;

use crate::errors::Result;
use crate::platform::PlatformUpdater;
use crate::structs::{InternalMessage, KeybindInfo};

enum LoopMessage {
    Activated(String),
    Deactivated(String),
    Update(Vec<KeybindInfo>),
}

pub fn start_keybinds(tx: Sender<InternalMessage>, app_id: Option<String>) -> PlatformUpdater {
    let (update_tx, update_rx) = futures::channel::mpsc::unbounded::<Vec<KeybindInfo>>();

    let tx_clone = tx;
    std::thread::spawn(move || {
        block_on(async {
            if let Err(e) = run_wayland_loop(tx_clone.clone(), update_rx, app_id).await {
                let _ = tx_clone.send(InternalMessage::FatalError(format!(
                    "Wayland backend error: {e}"
                )));
            }
        });
    });

    update_tx
}

async fn run_wayland_loop(
    tx: Sender<InternalMessage>,
    update_rx: futures::channel::mpsc::UnboundedReceiver<Vec<KeybindInfo>>,
    app_id: Option<String>,
) -> Result<()> {
    match app_id {
        Some(app_id_str) => {
            match AppID::from_str(&app_id_str) {
                Ok(id) => {
                    if let Err(err) = register_host_app(id).await {
                        eprintln!("Goofbind: Failed to register host app: {:?}", err);
                    } else {
                        eprintln!("Goofbind: Successfully registered host app with ID: {}", app_id_str);
                    }
                }
                Err(err) => {
                    eprintln!("Goofbind: Failed to parse AppID from '{}': {:?}", app_id_str, err);
                }
            }
        }
        None => {
            eprintln!("Goofbind: No app_id argument was passed to the binary.");
        }
    }

    let portal = GlobalShortcuts::new().await?;
    let session = portal
        .create_session(ashpd::desktop::CreateSessionOptions::default())
        .await?;

    let activated = portal.receive_activated().await?;
    let deactivated = portal.receive_deactivated().await?;

    let events = futures::stream::select(
        activated.map(|a| LoopMessage::Activated(a.shortcut_id().to_owned())),
        deactivated.map(|d| LoopMessage::Deactivated(d.shortcut_id().to_owned())),
    );

    let mut combined = futures::stream::select(events, update_rx.map(LoopMessage::Update));

    let mut current_keybinds: Vec<KeybindInfo> = Vec::new();

    while let Some(msg) = combined.next().await {
        match msg {
            LoopMessage::Activated(id) => {
                let _ = tx.send(InternalMessage::WaylandEvent { id, pressed: true });
            }
            LoopMessage::Deactivated(id) => {
                let _ = tx.send(InternalMessage::WaylandEvent { id, pressed: false });
            }
            LoopMessage::Update(keybinds) => {
                let needs_update = current_keybinds.len() != keybinds.len()
                    || current_keybinds
                        .iter()
                        .zip(keybinds.iter())
                        .any(|(a, b)| a.id != b.id || a.name != b.name);

                if needs_update {
                    current_keybinds = keybinds.clone();
                    let shortcuts: Vec<NewShortcut> = keybinds
                        .iter()
                        .map(|x| {
                            NewShortcut::new(&x.id, x.name.clone().unwrap_or_else(|| x.id.clone()))
                        })
                        .collect();

                    let _ = portal
                        .bind_shortcuts(
                            &session,
                            &shortcuts,
                            None,
                            ashpd::desktop::global_shortcuts::BindShortcutsOptions::default(),
                        )
                        .await;
                }
            }
        }
    }
    Ok(())
}
