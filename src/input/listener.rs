use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use super::InputEvent;

pub struct Listener {
    pub events: Receiver<InputEvent>,
    pub status: Receiver<Result<(), String>>,
    _thread: JoinHandle<()>,
}

pub fn start_listener() -> Listener {
    let (event_tx, events) = mpsc::channel();
    let (status_tx, status) = mpsc::channel();
    let thread_status_tx = status_tx.clone();
    let thread = thread::Builder::new()
        .name("key-jolt-input-listener".to_string())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                rdev::listen(move |event| {
                    if let Some(event) = map_event(event.event_type) {
                        let _ = event_tx.send(event);
                    }
                })
            }));
            let message = match result {
                Ok(Ok(())) => Err("The global input listener exited.".to_string()),
                Ok(Err(error)) => Err(format!("The global input listener failed: {error:?}")),
                Err(_) => Err("The global input listener stopped unexpectedly.".to_string()),
            };
            let _ = thread_status_tx.send(message);
        });
    match thread {
        Ok(thread) => Listener {
            events,
            status,
            _thread: thread,
        },
        Err(error) => {
            let _ = status_tx.send(Err(format!(
                "Could not start the global input listener: {error}"
            )));
            Listener {
                events,
                status,
                _thread: thread::spawn(|| {}),
            }
        }
    }
}

fn map_event(event: rdev::EventType) -> Option<InputEvent> {
    match event {
        rdev::EventType::KeyPress(key) => map_key(key).map(InputEvent::KeyPressed),
        rdev::EventType::KeyRelease(key) => map_key(key).map(InputEvent::KeyReleased),
        rdev::EventType::ButtonPress(button) => match button {
            rdev::Button::Left => Some(InputEvent::ButtonPressed("left".to_string())),
            rdev::Button::Right => Some(InputEvent::ButtonPressed("right".to_string())),
            rdev::Button::Middle => Some(InputEvent::ButtonPressed("middle_scroll".to_string())),
            rdev::Button::Unknown(_) => None,
        },
        rdev::EventType::Wheel { .. }
        | rdev::EventType::MouseMove { .. }
        | rdev::EventType::ButtonRelease(_) => None,
    }
}

fn map_key(key: rdev::Key) -> Option<String> {
    let name = match key {
        rdev::Key::Alt => "alt_left",
        rdev::Key::AltGr => "alt_right",
        rdev::Key::Backspace => "backspace",
        rdev::Key::CapsLock => "caps_lock",
        rdev::Key::NumLock => "num_lock",
        rdev::Key::ControlLeft => "ctrl_left",
        rdev::Key::ControlRight => "ctrl_right",
        rdev::Key::ShiftLeft => "shift_left",
        rdev::Key::ShiftRight => "shift_right",
        rdev::Key::MetaLeft => "win_left",
        rdev::Key::MetaRight => "win_right",
        rdev::Key::Delete => "delete",
        rdev::Key::DownArrow => "arrow_down",
        rdev::Key::End => "end",
        rdev::Key::Escape => "esc",
        rdev::Key::F1 => "f1",
        rdev::Key::F2 => "f2",
        rdev::Key::F3 => "f3",
        rdev::Key::F4 => "f4",
        rdev::Key::F5 => "f5",
        rdev::Key::F6 => "f6",
        rdev::Key::F7 => "f7",
        rdev::Key::F8 => "f8",
        rdev::Key::F9 => "f9",
        rdev::Key::F10 => "f10",
        rdev::Key::F11 => "f11",
        rdev::Key::F12 => "f12",
        rdev::Key::PrintScreen => "print_screen",
        rdev::Key::Home => "home",
        rdev::Key::LeftArrow => "arrow_left",
        rdev::Key::RightArrow => "arrow_right",
        rdev::Key::UpArrow => "arrow_up",
        rdev::Key::PageDown => "page_down",
        rdev::Key::PageUp => "page_up",
        rdev::Key::Return => "enter",
        rdev::Key::Space => "space",
        rdev::Key::Tab => "tab",
        rdev::Key::BackQuote => "grave",
        rdev::Key::Num1 => "1",
        rdev::Key::Num2 => "2",
        rdev::Key::Num3 => "3",
        rdev::Key::Num4 => "4",
        rdev::Key::Num5 => "5",
        rdev::Key::Num6 => "6",
        rdev::Key::Num7 => "7",
        rdev::Key::Num8 => "8",
        rdev::Key::Num9 => "9",
        rdev::Key::Num0 => "0",
        rdev::Key::Minus => "minus",
        rdev::Key::Equal => "equal",
        rdev::Key::KeyQ => "q",
        rdev::Key::KeyW => "w",
        rdev::Key::KeyE => "e",
        rdev::Key::KeyR => "r",
        rdev::Key::KeyT => "t",
        rdev::Key::KeyY => "y",
        rdev::Key::KeyU => "u",
        rdev::Key::KeyI => "i",
        rdev::Key::KeyO => "o",
        rdev::Key::KeyP => "p",
        rdev::Key::LeftBracket => "left_bracket",
        rdev::Key::RightBracket => "right_bracket",
        rdev::Key::BackSlash => "backslash",
        rdev::Key::KeyA => "a",
        rdev::Key::KeyS => "s",
        rdev::Key::KeyD => "d",
        rdev::Key::KeyF => "f",
        rdev::Key::KeyG => "g",
        rdev::Key::KeyH => "h",
        rdev::Key::KeyJ => "j",
        rdev::Key::KeyK => "k",
        rdev::Key::KeyL => "l",
        rdev::Key::SemiColon => "semicolon",
        rdev::Key::Quote => "quote",
        rdev::Key::KeyZ => "z",
        rdev::Key::KeyX => "x",
        rdev::Key::KeyC => "c",
        rdev::Key::KeyV => "v",
        rdev::Key::KeyB => "b",
        rdev::Key::KeyN => "n",
        rdev::Key::KeyM => "m",
        rdev::Key::Comma => "comma",
        rdev::Key::Dot => "period",
        rdev::Key::Slash => "slash",
        rdev::Key::Insert => "insert",
        rdev::Key::KpReturn => "num_enter_1",
        rdev::Key::KpMinus => "num_minus",
        rdev::Key::KpPlus => "num_plus_1",
        rdev::Key::KpMultiply => "num_multiply",
        rdev::Key::KpDivide => "num_divide",
        rdev::Key::Kp0 => "num_0",
        rdev::Key::Kp1 => "num_1",
        rdev::Key::Kp2 => "num_2",
        rdev::Key::Kp3 => "num_3",
        rdev::Key::Kp4 => "num_4",
        rdev::Key::Kp5 => "num_5",
        rdev::Key::Kp6 => "num_6",
        rdev::Key::Kp7 => "num_7",
        rdev::Key::Kp8 => "num_8",
        rdev::Key::Kp9 => "num_9",
        rdev::Key::KpDelete => "num_decimal",
        _ => return None,
    };
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_modifier_and_print_screen_keys() {
        assert_eq!(map_key(rdev::Key::Alt), Some("alt_left".to_string()));
        assert_eq!(map_key(rdev::Key::AltGr), Some("alt_right".to_string()));
        assert_eq!(
            map_key(rdev::Key::ControlLeft),
            Some("ctrl_left".to_string())
        );
        assert_eq!(
            map_key(rdev::Key::ControlRight),
            Some("ctrl_right".to_string())
        );
        assert_eq!(
            map_key(rdev::Key::ShiftLeft),
            Some("shift_left".to_string())
        );
        assert_eq!(
            map_key(rdev::Key::ShiftRight),
            Some("shift_right".to_string())
        );
        assert_eq!(map_key(rdev::Key::MetaLeft), Some("win_left".to_string()));
        assert_eq!(map_key(rdev::Key::MetaRight), Some("win_right".to_string()));
        assert_eq!(
            map_key(rdev::Key::PrintScreen),
            Some("print_screen".to_string())
        );
    }
}
