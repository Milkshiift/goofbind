use rdev::{Event, EventType, Key, listen};
use std::sync::OnceLock;
use std::sync::mpsc::Sender;

use crate::platform::PlatformUpdater;
use crate::structs::InternalMessage;

static X11_TX: OnceLock<Sender<InternalMessage>> = OnceLock::new();

pub fn start_keybinds(tx: Sender<InternalMessage>) -> Option<PlatformUpdater> {
    let _ = X11_TX.set(tx.clone());
    std::thread::spawn(move || {
        if let Err(e) = listen(x11_callback) {
            let _ = tx.send(InternalMessage::FatalError(format!(
                "X11 listener failed: {e:?}"
            )));
        }
    });
    None
}

#[allow(clippy::needless_pass_by_value)] // Allowed because `rdev::listen` requires a callback signature taking `Event` by value.
fn x11_callback(event: Event) {
    let (is_pressed, key) = match event.event_type {
        EventType::KeyPress(k) => (true, k),
        EventType::KeyRelease(k) => (false, k),
        _ => return,
    };

    if let Some(vk) = rdev_key_to_vk(key)
        && let Some(tx) = X11_TX.get()
    {
        let _ = tx.send(InternalMessage::RawKey {
            keycode: vk,
            pressed: is_pressed,
        });
    }
}

const fn rdev_key_to_vk(key: Key) -> Option<u32> {
    match key {
        Key::KeyA => Some(65),
        Key::KeyB => Some(66),
        Key::KeyC => Some(67),
        Key::KeyD => Some(68),
        Key::KeyE => Some(69),
        Key::KeyF => Some(70),
        Key::KeyG => Some(71),
        Key::KeyH => Some(72),
        Key::KeyI => Some(73),
        Key::KeyJ => Some(74),
        Key::KeyK => Some(75),
        Key::KeyL => Some(76),
        Key::KeyM => Some(77),
        Key::KeyN => Some(78),
        Key::KeyO => Some(79),
        Key::KeyP => Some(80),
        Key::KeyQ => Some(81),
        Key::KeyR => Some(82),
        Key::KeyS => Some(83),
        Key::KeyT => Some(84),
        Key::KeyU => Some(85),
        Key::KeyV => Some(86),
        Key::KeyW => Some(87),
        Key::KeyX => Some(88),
        Key::KeyY => Some(89),
        Key::KeyZ => Some(90),

        Key::Num1 => Some(49),
        Key::Num2 => Some(50),
        Key::Num3 => Some(51),
        Key::Num4 => Some(52),
        Key::Num5 => Some(53),
        Key::Num6 => Some(54),
        Key::Num7 => Some(55),
        Key::Num8 => Some(56),
        Key::Num9 => Some(57),
        Key::Num0 => Some(48),

        Key::Kp1 => Some(97),
        Key::Kp2 => Some(98),
        Key::Kp3 => Some(99),
        Key::Kp4 => Some(100),
        Key::Kp5 => Some(101),
        Key::Kp6 => Some(102),
        Key::Kp7 => Some(103),
        Key::Kp8 => Some(104),
        Key::Kp9 => Some(105),
        Key::Kp0 => Some(96),

        Key::F1 => Some(112),
        Key::F2 => Some(113),
        Key::F3 => Some(114),
        Key::F4 => Some(115),
        Key::F5 => Some(116),
        Key::F6 => Some(117),
        Key::F7 => Some(118),
        Key::F8 => Some(119),
        Key::F9 => Some(120),
        Key::F10 => Some(121),
        Key::F11 => Some(122),
        Key::F12 => Some(123),

        Key::Return | Key::KpReturn => Some(13),
        Key::Escape => Some(27),
        Key::Backspace => Some(8),
        Key::Tab => Some(9),
        Key::Space => Some(32),

        Key::Minus => Some(189),
        Key::Equal => Some(187),
        Key::LeftBracket => Some(219),
        Key::RightBracket => Some(221),
        Key::BackQuote => Some(192),
        Key::SemiColon => Some(186),
        Key::Quote => Some(222),
        Key::BackSlash => Some(220),
        Key::Comma => Some(188),
        Key::Dot => Some(190),
        Key::Slash => Some(191),

        Key::UpArrow => Some(38),
        Key::DownArrow => Some(40),
        Key::LeftArrow => Some(37),
        Key::RightArrow => Some(39),
        Key::Home => Some(36),
        Key::End => Some(35),
        Key::PageUp => Some(33),
        Key::PageDown => Some(34),
        Key::Insert => Some(45),
        Key::Delete => Some(46),

        Key::CapsLock => Some(20),
        Key::PrintScreen => Some(44),
        Key::ScrollLock => Some(145),
        Key::Pause => Some(19),
        Key::NumLock => Some(144),

        Key::KpMinus => Some(109),
        Key::KpPlus => Some(107),
        Key::KpMultiply => Some(106),
        Key::KpDivide => Some(111),

        Key::ShiftLeft | Key::ShiftRight => Some(16),
        Key::ControlLeft | Key::ControlRight => Some(17),
        Key::Alt | Key::AltGr => Some(18),
        Key::MetaLeft | Key::MetaRight => Some(91),

        _ => None,
    }
}
