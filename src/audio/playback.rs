use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;

use rodio::{Decoder, DeviceSinkBuilder, Player, Source, buffer::SamplesBuffer};

use crate::activity::{ACTIVITY, ActivityChannel};
use crate::presets::{self, PlaybackMode, PresetKind};
use crate::settings::store::AppSettings;

enum Command {
    Reload(PresetKind, String),
    Play(String, bool),
    Enabled(bool),
    Stop,
}

pub struct PlaybackHandle {
    sender: Sender<Command>,
    pub app_enabled: Arc<AtomicBool>,
    keyboard_volume: Arc<AtomicU32>,
    mouse_volume: Arc<AtomicU32>,
}

#[derive(Clone)]
pub struct PlaybackSender(Sender<Command>);

impl PlaybackSender {
    pub fn play(&self, identifier: &str) {
        let mouse = matches!(identifier, "left" | "right" | "middle_scroll");
        let _ = self.0.send(Command::Play(
            binding_key(
                if mouse {
                    PresetKind::Mouse
                } else {
                    PresetKind::Keyboard
                },
                identifier,
            ),
            mouse,
        ));
    }

    pub fn set_enabled(&self, enabled: bool) {
        let _ = self.0.send(Command::Enabled(enabled));
    }
}

impl PlaybackHandle {
    pub fn new(root: PathBuf, settings: &AppSettings, status: Sender<String>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let app_enabled = Arc::new(AtomicBool::new(settings.app_enabled));
        let keyboard_volume = Arc::new(AtomicU32::new(
            settings.effective_keyboard_volume().to_bits(),
        ));
        let mouse_volume = Arc::new(AtomicU32::new(settings.effective_mouse_volume().to_bits()));
        let worker_keyboard_volume = keyboard_volume.clone();
        let worker_mouse_volume = mouse_volume.clone();
        let worker_enabled = app_enabled.clone();
        let worker_status = status.clone();
        if let Err(error) = thread::Builder::new()
            .name("key-jolt-audio".to_string())
            .spawn(move || {
                let output = match DeviceSinkBuilder::open_default_sink() {
                    Ok(output) => output,
                    Err(error) => {
                        let _ = worker_status
                            .send(format!("Could not open an audio output device: {error}"));
                        return;
                    }
                };
                let mut bindings = HashMap::<String, RuntimeBinding>::new();
                let mut cursors = HashMap::<String, usize>::new();
                let mut active_players = Vec::<Player>::new();
                let mut random_state = 0x9e3779b97f4a7c15_u64;
                while let Ok(command) = receiver.recv() {
                    active_players.retain(|player| !player.empty());
                    match command {
                        Command::Reload(kind, id) => {
                            match presets::runtime_bindings(&root, kind, &id) {
                                Ok(raw) => {
                                    bindings.retain(|key, _| {
                                        !key.starts_with(if kind == PresetKind::Keyboard {
                                            "k:"
                                        } else {
                                            "m:"
                                        })
                                    });
                                    for (identifier, (mode, paths)) in raw {
                                        let decoded = paths
                                            .into_iter()
                                            .filter_map(|path| decode(&path).ok())
                                            .collect::<Vec<_>>();
                                        if !decoded.is_empty() {
                                            let key = binding_key(kind, &identifier);
                                            bindings.insert(
                                                key,
                                                RuntimeBinding {
                                                    mode,
                                                    sounds: decoded,
                                                },
                                            );
                                        }
                                    }
                                    cursors.clear();
                                }
                                Err(error) => {
                                    let _ = worker_status
                                        .send(format!("Could not load active sounds: {error}"));
                                }
                            }
                        }
                        Command::Play(identifier, mouse) => {
                            if !worker_enabled.load(Ordering::Acquire) {
                                continue;
                            }
                            let alias = alias_binding_key(&identifier);
                            if let Some(binding) = bindings
                                .get(&identifier)
                                .or_else(|| alias.as_ref().and_then(|key| bindings.get(key)))
                            {
                                let index = match binding.mode {
                                    PlaybackMode::Sequential => {
                                        let cursor = cursors.entry(identifier.clone()).or_default();
                                        choose_index(binding.mode, binding.sounds.len(), cursor, 0)
                                            .unwrap()
                                    }
                                    PlaybackMode::Random => {
                                        random_state ^= random_state << 13;
                                        random_state ^= random_state >> 7;
                                        random_state ^= random_state << 17;
                                        choose_index(
                                            binding.mode,
                                            binding.sounds.len(),
                                            &mut 0,
                                            random_state,
                                        )
                                        .unwrap()
                                    }
                                };
                                let volume = if mouse {
                                    worker_mouse_volume.load(Ordering::Acquire)
                                } else {
                                    worker_keyboard_volume.load(Ordering::Acquire)
                                };
                                let player = Player::connect_new(output.mixer());
                                player.set_volume(f32::from_bits(volume).clamp(0.0, 2.0));
                                player.append(binding.sounds[index].clone());
                                ACTIVITY.record_sound(
                                    if mouse {
                                        ActivityChannel::Mouse
                                    } else {
                                        ActivityChannel::Keyboard
                                    },
                                    f32::from_bits(volume).clamp(0.0, 2.0),
                                );
                                active_players.push(player);
                            }
                        }
                        Command::Enabled(false) => {
                            for player in &active_players {
                                player.set_volume(0.0);
                                player.stop();
                            }
                            active_players.clear();
                        }
                        Command::Enabled(true) => {}
                        Command::Stop => break,
                    }
                }
            })
        {
            let _ = status.send(format!("Could not start the audio worker: {error}"));
        }
        Self {
            sender,
            app_enabled,
            keyboard_volume,
            mouse_volume,
        }
    }

    pub fn update_settings(&self, settings: &AppSettings) {
        let was_enabled = self
            .app_enabled
            .swap(settings.app_enabled, Ordering::AcqRel);
        if was_enabled != settings.app_enabled {
            self.sender_handle().set_enabled(settings.app_enabled);
        }
        self.keyboard_volume.store(
            settings.effective_keyboard_volume().to_bits(),
            Ordering::Release,
        );
        self.mouse_volume.store(
            settings.effective_mouse_volume().to_bits(),
            Ordering::Release,
        );
    }

    pub fn reload(&self, kind: PresetKind, id: impl Into<String>) {
        let _ = self.sender.send(Command::Reload(kind, id.into()));
    }

    pub fn sender_handle(&self) -> PlaybackSender {
        PlaybackSender(self.sender.clone())
    }
}

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Stop);
    }
}

struct RuntimeBinding {
    mode: PlaybackMode,
    sounds: Vec<SamplesBuffer>,
}

fn binding_key(kind: PresetKind, identifier: &str) -> String {
    format!(
        "{}:{identifier}",
        if kind == PresetKind::Keyboard {
            "k"
        } else {
            "m"
        }
    )
}

fn alias_binding_key(identifier: &str) -> Option<String> {
    let (prefix, key) = identifier.split_once(':')?;
    let alias = match key {
        "shift_left" => "left_shift",
        "left_shift" => "shift_left",
        "shift_right" => "right_shift",
        "right_shift" => "shift_right",
        "ctrl_left" => "left_ctrl",
        "left_ctrl" => "ctrl_left",
        "ctrl_right" => "right_ctrl",
        "right_ctrl" => "ctrl_right",
        "win_left" => "left_meta",
        "left_meta" => "win_left",
        "win_right" => "right_meta",
        "right_meta" => "win_right",
        "num_enter_1" => "num_enter_2",
        "num_enter_2" => "num_enter_1",
        "num_plus_1" => "num_plus_2",
        "num_plus_2" => "num_plus_1",
        _ => return None,
    };
    Some(format!("{prefix}:{alias}"))
}

fn decode(path: &PathBuf) -> Result<SamplesBuffer, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let decoder = Decoder::try_from(file).map_err(|error| error.to_string())?;
    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    Ok(SamplesBuffer::new(
        channels,
        sample_rate,
        decoder.collect::<Vec<_>>(),
    ))
}

fn choose_index(
    mode: PlaybackMode,
    length: usize,
    cursor: &mut usize,
    random: u64,
) -> Option<usize> {
    if length == 0 {
        return None;
    }
    match mode {
        PlaybackMode::Sequential => {
            let selected = *cursor % length;
            *cursor = cursor.wrapping_add(1);
            Some(selected)
        }
        PlaybackMode::Random => Some(random as usize % length),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_selection_cycles_over_enabled_sounds() {
        let mut cursor = 0;
        let selected = (0..5)
            .map(|_| choose_index(PlaybackMode::Sequential, 3, &mut cursor, 0).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec![0, 1, 2, 0, 1]);
    }

    #[test]
    fn random_selection_uses_a_sound_from_the_available_pool() {
        let selected = choose_index(PlaybackMode::Random, 4, &mut 0, 7).unwrap();
        assert_eq!(selected, 3);
        assert_eq!(choose_index(PlaybackMode::Random, 0, &mut 0, 7), None);
    }
}
