use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputAction {
    Sound(String),
    ToggleEnabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    KeyPressed(String),
    KeyReleased(String),
    ButtonPressed(String),
}

const FUNCTION_KEY_DEBOUNCE: Duration = Duration::from_millis(40);

pub struct InputState {
    held: HashSet<String>,
    last_function_press: HashMap<String, Instant>,
    alt_left: bool,
    alt_right: bool,
    shift_left: bool,
    shift_right: bool,
    shortcut_held: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            held: HashSet::new(),
            last_function_press: HashMap::new(),
            alt_left: false,
            alt_right: false,
            shift_left: false,
            shift_right: false,
            shortcut_held: false,
        }
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<InputAction> {
        self.handle_at(event, Instant::now())
    }

    fn handle_at(&mut self, event: InputEvent, now: Instant) -> Option<InputAction> {
        match event {
            InputEvent::KeyPressed(key) => {
                if !self.held.insert(key.clone()) {
                    return None;
                }

                if is_function_key(&key) {
                    let duplicate = self
                        .last_function_press
                        .get(&key)
                        .is_some_and(|last| now.duration_since(*last) < FUNCTION_KEY_DEBOUNCE);
                    self.last_function_press.insert(key.clone(), now);
                    if duplicate {
                        return None;
                    }
                }

                self.set_modifier(&key, true);
                let shortcut = (self.alt_left || self.alt_right)
                    && (self.shift_left || self.shift_right)
                    && key == "m";
                if shortcut && !self.shortcut_held {
                    self.shortcut_held = true;
                    return Some(InputAction::ToggleEnabled);
                }
                self.shortcut_held = shortcut;
                Some(InputAction::Sound(key))
            }
            InputEvent::KeyReleased(key) => {
                self.held.remove(&key);
                self.set_modifier(&key, false);
                if key == "m"
                    || !(self.alt_left || self.alt_right)
                    || !(self.shift_left || self.shift_right)
                {
                    self.shortcut_held = false;
                }
                None
            }
            InputEvent::ButtonPressed(button) => Some(InputAction::Sound(button)),
        }
    }

    fn set_modifier(&mut self, key: &str, pressed: bool) {
        match key {
            "alt_left" => self.alt_left = pressed,
            "alt_right" => self.alt_right = pressed,
            "shift_left" => self.shift_left = pressed,
            "shift_right" => self.shift_right = pressed,
            _ => {}
        }
    }
}

fn is_function_key(key: &str) -> bool {
    matches!(
        key,
        "f1" | "f2" | "f3" | "f4" | "f5" | "f6" | "f7" | "f8" | "f9" | "f10" | "f11" | "f12"
    )
}

pub struct Throttle {
    last: Option<Instant>,
    interval: Duration,
}

impl Throttle {
    pub fn new(interval: Duration) -> Self {
        Self {
            last: None,
            interval,
        }
    }

    pub fn allow(&mut self, now: Instant) -> bool {
        if self
            .last
            .is_some_and(|last| now.duration_since(last) < self.interval)
        {
            return false;
        }
        self.last = Some(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_keys_suppress_repeats_and_release_allows_next_press() {
        let mut state = InputState::new();
        assert_eq!(
            state.handle(InputEvent::KeyPressed("a".into())),
            Some(InputAction::Sound("a".into()))
        );
        assert_eq!(state.handle(InputEvent::KeyPressed("a".into())), None);
        assert_eq!(state.handle(InputEvent::KeyReleased("a".into())), None);
        assert_eq!(
            state.handle(InputEvent::KeyPressed("a".into())),
            Some(InputAction::Sound("a".into()))
        );
    }

    #[test]
    fn modifier_keys_produce_sounds() {
        let mut state = InputState::new();
        for key in [
            "alt_left",
            "alt_right",
            "shift_left",
            "shift_right",
            "ctrl_left",
            "ctrl_right",
            "win_left",
            "win_right",
        ] {
            assert_eq!(
                state.handle(InputEvent::KeyPressed(key.into())),
                Some(InputAction::Sound(key.into()))
            );
            assert_eq!(state.handle(InputEvent::KeyReleased(key.into())), None);
        }
    }

    #[test]
    fn alt_shift_m_toggles_once_and_accepts_either_modifier_side() {
        let mut state = InputState::new();
        state.handle(InputEvent::KeyPressed("alt_right".into()));
        state.handle(InputEvent::KeyPressed("shift_left".into()));
        assert_eq!(
            state.handle(InputEvent::KeyPressed("m".into())),
            Some(InputAction::ToggleEnabled)
        );
        assert_eq!(state.handle(InputEvent::KeyPressed("m".into())), None);
    }

    #[test]
    fn function_keys_suppress_duplicate_press_release_pairs() {
        let mut state = InputState::new();
        let start = Instant::now();

        assert_eq!(
            state.handle_at(InputEvent::KeyPressed("f1".into()), start),
            Some(InputAction::Sound("f1".into()))
        );
        assert_eq!(
            state.handle_at(
                InputEvent::KeyReleased("f1".into()),
                start + Duration::from_millis(2),
            ),
            None
        );
        assert_eq!(
            state.handle_at(
                InputEvent::KeyPressed("f1".into()),
                start + Duration::from_millis(5),
            ),
            None
        );
        assert_eq!(
            state.handle_at(
                InputEvent::KeyReleased("f1".into()),
                start + Duration::from_millis(7),
            ),
            None
        );
        assert_eq!(
            state.handle_at(
                InputEvent::KeyPressed("f1".into()),
                start + Duration::from_millis(50),
            ),
            Some(InputAction::Sound("f1".into()))
        );
    }

    #[test]
    fn throttle_blocks_events_inside_interval() {
        let start = Instant::now();
        let mut throttle = Throttle::new(Duration::from_millis(20));
        assert!(throttle.allow(start));
        assert!(!throttle.allow(start + Duration::from_millis(19)));
        assert!(throttle.allow(start + Duration::from_millis(20)));
    }
}
