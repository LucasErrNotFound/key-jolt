use super::key_identifier::runtime_key_identifier;
use super::model::{
    Binding, KeyboardPresetData, KeyboardPresetState, MousePresetData, PlaybackMode, PresetKind,
};

use super::paths::safe_relative_path;
use super::repository::load_preset_source;
use super::sounds::copy_sound;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) struct KeyboardBindingMaps {
    pub(super) layouts: [BTreeMap<String, Binding>; 3],
    pub(super) independent_layouts: Option<[BTreeMap<String, Binding>; 3]>,
}

pub(crate) fn runtime_bindings(
    root: &Path,
    kind: PresetKind,
    id: &str,
) -> Result<BTreeMap<String, (PlaybackMode, Vec<PathBuf>)>, String> {
    if root.as_os_str().is_empty() || id.is_empty() {
        return Ok(BTreeMap::new());
    }

    let Some((preset, directory)) = load_preset_source(root, kind, id)? else {
        return Ok(BTreeMap::new());
    };

    let mut bindings = BTreeMap::new();
    for (identifier, binding) in preset.bindings {
        let sounds = binding
            .sounds
            .into_iter()
            .filter(|sound| sound.enabled)
            .filter_map(|sound| {
                safe_relative_path(&sound.file).map(|relative| directory.join(relative))
            })
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        if !sounds.is_empty() {
            bindings.insert(
                runtime_key_identifier(&identifier).to_string(),
                (binding.playback_mode, sounds),
            );
        }
    }
    Ok(bindings)
}

pub(super) fn keyboard_preset_state(
    data: &KeyboardPresetData,
    mut mappings: KeyboardBindingMaps,
) -> KeyboardPresetState {
    let active = data.layout_index.min(2);
    if let Some(independent) = &mut mappings.independent_layouts {
        for (index, bindings) in independent.iter_mut().enumerate() {
            if index == active {
                *bindings = mappings.layouts[active].clone();
            } else {
                bindings.retain(|key, binding| mappings.layouts[active].get(key) != Some(binding));
            }
        }
    }
    KeyboardPresetState {
        layout_index: active,
        sync_selections: data.sync_selections,
        selected_keys: data
            .selected_keys
            .clone()
            .map(|keys| keys.into_iter().map(str::to_string).collect::<Vec<_>>()),
        layout_bindings: Some(mappings.layouts),
        independent_layout_bindings: mappings.independent_layouts,
    }
}

pub(super) fn keyboard_bindings(
    data: &KeyboardPresetData,
    directory: &Path,
) -> Result<KeyboardBindingMaps, String> {
    let mut layouts: [BTreeMap<String, Binding>; 3] = std::array::from_fn(|_| BTreeMap::new());
    let mut independent_layouts = (data.sync_selections
        && data
            .files
            .iter()
            .any(|sound| sound.independent_assigned_keys.is_some()))
    .then(|| std::array::from_fn(|_| BTreeMap::<String, Binding>::new()));
    for sound in &data.files {
        let independent = sound
            .independent_assigned_keys
            .as_ref()
            .unwrap_or(&sound.assigned_keys);
        if sound.assigned_keys.iter().all(Vec::is_empty) && independent.iter().all(Vec::is_empty) {
            continue;
        }
        let reference = copy_sound(&sound.path, &sound.name, directory)?;
        for (bindings, keys) in layouts.iter_mut().zip(&sound.assigned_keys).chain(
            independent_layouts
                .iter_mut()
                .flat_map(|layouts| layouts.iter_mut().zip(independent)),
        ) {
            for key in keys {
                let binding = bindings
                    .entry(runtime_key_identifier(key).to_string())
                    .or_insert_with(|| Binding {
                        sounds: Vec::new(),
                        playback_mode: if data.random_playback {
                            PlaybackMode::Random
                        } else {
                            PlaybackMode::Sequential
                        },
                    });
                if !binding.sounds.contains(&reference) {
                    binding.sounds.push(reference.clone());
                }
            }
        }
    }
    Ok(KeyboardBindingMaps {
        layouts,
        independent_layouts,
    })
}

pub(super) fn mouse_bindings(
    data: &MousePresetData,
    directory: &Path,
) -> Result<BTreeMap<String, Binding>, String> {
    let copied = data
        .files
        .iter()
        .map(|sound| {
            let path = if sound.assigned_buttons.iter().any(|assigned| *assigned) {
                Some(copy_sound(&sound.path, &sound.name, directory)?)
            } else {
                None
            };
            Ok::<_, String>(path)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut bindings = BTreeMap::new();
    for (index, button) in ["left", "right", "middle_scroll"].iter().enumerate() {
        let sounds = data
            .files
            .iter()
            .zip(&copied)
            .filter_map(|(sound, reference)| {
                sound.assigned_buttons[index]
                    .then(|| reference.clone())
                    .flatten()
            })
            .collect::<Vec<_>>();
        if !sounds.is_empty() {
            bindings.insert(
                (*button).to_string(),
                Binding {
                    sounds,
                    playback_mode: if data.random_playback {
                        PlaybackMode::Random
                    } else {
                        PlaybackMode::Sequential
                    },
                },
            );
        }
    }
    Ok(bindings)
}
