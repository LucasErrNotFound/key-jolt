use crate::input::{InputAction, InputState, Listener, Throttle};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

pub(crate) enum RuntimeEvent {
    ToggleApp(bool),
    Warning(String),
    TrayOpen,
    TrayExit,
}

pub(crate) fn start_runtime_thread(
    listener: Listener,
    playback: crate::audio::playback::PlaybackSender,
    enabled: Arc<AtomicBool>,
    events: Sender<RuntimeEvent>,
    audio_status: Receiver<String>,
) -> JoinHandle<()> {
    let thread_events = events.clone();
    std::thread::Builder::new()
        .name("key-jolt-input-controller".to_string())
        .spawn(move || {
            let Listener {
                events: input_events,
                status: listener_status,
                ..
            } = listener;
            let mut state = InputState::new();
            let mut mouse_throttle = Throttle::new(Duration::from_millis(12));
            let mut listener_running = true;
            while listener_running {
                match input_events.recv_timeout(Duration::from_millis(25)) {
                    Ok(event) => match state.handle(event) {
                        Some(InputAction::Sound(identifier)) if enabled.load(Ordering::Acquire) => {
                            if !matches!(identifier.as_str(), "left" | "right" | "middle_scroll")
                                || mouse_throttle.allow(std::time::Instant::now())
                            {
                                playback.play(&identifier)
                            }
                        }
                        Some(InputAction::ToggleEnabled) => {
                            let value = !enabled.load(Ordering::Acquire);
                            enabled.store(value, Ordering::Release);
                            playback.set_enabled(value);
                            let _ = thread_events.send(RuntimeEvent::ToggleApp(value));
                        }
                        _ => {}
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => listener_running = false,
                }
                for result in listener_status.try_iter() {
                    if let Err(message) = result {
                        let _ = thread_events.send(RuntimeEvent::Warning(message));
                        listener_running = false;
                    }
                }
                for message in audio_status.try_iter() {
                    let _ = thread_events.send(RuntimeEvent::Warning(message));
                }
            }
        })
        .unwrap_or_else(|error| {
            let _ = events.send(RuntimeEvent::Warning(format!(
                "Could not start input processing: {error}"
            )));
            std::thread::spawn(|| {})
        })
}
