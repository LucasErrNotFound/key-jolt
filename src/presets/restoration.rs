use super::key_identifier::{canonical_key_identifier, key_identifier};

use super::model::{
    Binding, KeyboardPresetData, KeyboardSoundData, LoadedPreset, MousePresetData, MouseSoundData,
    PlaybackMode, PresetFile, PresetKind, PresetWarning,
};

use std::collections::{BTreeMap, BTreeSet};

use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn loaded_preset(
    preset: PresetFile,
    kind: PresetKind,
    directory: &Path,
    warnings: &mut Vec<PresetWarning>,
) -> LoadedPreset {
    let layout_bindings = restore_layout_bindings(&preset);
    let bindings = if kind == PresetKind::Keyboard {
        layout_bindings
            .iter()
            .flat_map(|layout| layout.iter())
            .collect::<Vec<_>>()
    } else {
        preset.bindings.iter().collect::<Vec<_>>()
    };
    let mut available = BTreeMap::<String, PathBuf>::new();
    for (identifier, binding) in &bindings {
        for sound in &binding.sounds {
            if !sound.enabled {
                continue;
            }
            let relative = Path::new(&sound.file);
            let path = directory.join(relative);
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
                || !path.starts_with(directory)
                || !path.is_file()
            {
                warnings.push(PresetWarning {
                    message: format!(
                        "Preset '{}' has a missing sound for {identifier}.",
                        preset.name
                    ),
                });
                continue;
            }
            available.insert(sound.file.clone(), path);
        }
    }
    let keyboard = (kind == PresetKind::Keyboard).then(|| {
        let (selected_keys, layout_index, sync_selections) = restore_keyboard_selection(&preset);
        let mut selected_sound_names = BTreeMap::<String, String>::new();
        for (_, binding) in &bindings {
            for sound in &binding.sounds {
                if sound.enabled && available.contains_key(&sound.file) {
                    selected_sound_names.insert(sound.file.clone(), sound.name.clone());
                }
            }
        }
        let files = selected_sound_names
            .into_iter()
            .filter_map(|(file, name)| {
                let path = available.get(&file)?.clone();
                Some(KeyboardSoundData {
                    size: file_size(&path),
                    path,
                    name: name.into(),
                    assigned_keys: std::array::from_fn(|index| {
                        layout_bindings[index]
                            .iter()
                            .filter(|(_, binding)| {
                                binding
                                    .sounds
                                    .iter()
                                    .any(|sound| sound.enabled && sound.file == file)
                            })
                            .map(|(key, _)| canonical_key_identifier(key).to_string())
                            .collect::<BTreeSet<_>>()
                            .into_iter()
                            .collect()
                    }),
                })
            })
            .collect();
        KeyboardPresetData {
            selected_keys,
            files,
            random_playback: layout_bindings
                .iter()
                .flat_map(|layout| layout.values())
                .any(|binding| binding.playback_mode == PlaybackMode::Random),
            layout_index,
            sync_selections,
        }
    });
    let mouse = (kind == PresetKind::Mouse).then(|| {
        let identifiers = ["left", "right", "middle_scroll"];
        let mut files = BTreeMap::<String, MouseSoundData>::new();
        let mut modes = Vec::new();
        for (index, identifier) in identifiers.iter().enumerate() {
            if let Some(binding) = preset.bindings.get(*identifier) {
                modes.push(binding.playback_mode);
                for sound in &binding.sounds {
                    if !sound.enabled || !available.contains_key(&sound.file) {
                        continue;
                    }
                    let entry = files
                        .entry(sound.file.clone())
                        .or_insert_with(|| MouseSoundData {
                            path: available[&sound.file].clone(),
                            name: sound.name.clone().into(),
                            size: file_size(&available[&sound.file]),
                            assigned_buttons: [false; 3],
                        });
                    entry.assigned_buttons[index] = true;
                }
            }
        }
        MousePresetData {
            files: files.into_values().collect(),
            random_playback: modes.contains(&PlaybackMode::Random),
        }
    });
    let summary = if kind == PresetKind::Keyboard {
        let mapped_key_count = keyboard_mapped_key_count(&preset.bindings);
        let mapped_key_label = if mapped_key_count == 1 { "key" } else { "keys" };
        format!("Custom · {mapped_key_count} {mapped_key_label} mapped")
    } else {
        preset.summary.clone()
    };
    LoadedPreset {
        id: preset.id,
        name: preset.name,
        summary,
        kind,
        keyboard,
        mouse,
    }
}

fn restore_layout_bindings(preset: &PresetFile) -> [BTreeMap<String, Binding>; 3] {
    if let Some(layouts) = preset
        .keyboard
        .as_ref()
        .and_then(|state| state.layout_bindings.as_ref())
    {
        return layouts.clone();
    }
    let (_, active, sync) = restore_keyboard_selection(preset);
    std::array::from_fn(|index| {
        if index == active {
            preset.bindings.clone()
        } else if sync {
            preset
                .bindings
                .iter()
                .filter(|(key, _)| key_identifier_for_layout(key, index).is_some())
                .map(|(key, binding)| (key.clone(), binding.clone()))
                .collect()
        } else {
            BTreeMap::new()
        }
    })
}

pub(super) fn file_size(path: &Path) -> Option<gpui_kit::SharedString> {
    let bytes = fs::metadata(path).ok()?.len();
    let value = bytes as f64;
    let formatted = if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", value / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", value / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", value / (1024.0 * 1024.0 * 1024.0))
    };
    Some(formatted.into())
}

pub(super) fn keyboard_mapped_key_count(bindings: &BTreeMap<String, Binding>) -> usize {
    bindings
        .keys()
        .filter(|identifier| key_identifier(identifier).is_some())
        .map(|identifier| canonical_key_identifier(identifier).to_string())
        .collect::<BTreeSet<_>>()
        .len()
}

pub(super) fn restore_keyboard_selection(
    preset: &PresetFile,
) -> ([Vec<&'static str>; 3], usize, bool) {
    if let Some(state) = &preset.keyboard {
        let selected_keys = std::array::from_fn(|index| {
            let mut keys = Vec::new();
            for identifier in &state.selected_keys[index] {
                if let Some(key) = key_identifier(identifier)
                    && !keys.contains(&key)
                {
                    keys.push(key);
                }
            }
            keys
        });
        return (
            selected_keys,
            state.layout_index.min(2),
            state.sync_selections,
        );
    }

    let layout_index = infer_legacy_keyboard_layout(&preset.bindings);
    let mut selected_keys: [Vec<&'static str>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for identifier in preset.bindings.keys() {
        if let Some(key) = key_identifier_for_layout(identifier, layout_index)
            && !selected_keys[layout_index].contains(&key)
        {
            selected_keys[layout_index].push(key);
        }
    }
    (selected_keys, layout_index, false)
}

pub(super) fn infer_legacy_keyboard_layout(bindings: &BTreeMap<String, Binding>) -> usize {
    if bindings
        .keys()
        .any(|identifier| identifier.starts_with("num_"))
    {
        return 0;
    }

    const TKL_ONLY: &[&str] = &[
        "left_shift",
        "right_shift",
        "left_ctrl",
        "right_ctrl",
        "left_meta",
        "right_meta",
        "left_alt",
        "right_alt",
        "insert",
        "home",
        "page_up",
        "delete",
        "end",
        "page_down",
        "arrow_up",
        "arrow_left",
        "arrow_down",
        "arrow_right",
    ];
    if bindings
        .keys()
        .any(|identifier| TKL_ONLY.contains(&identifier.as_str()))
    {
        return 1;
    }

    2
}

pub(super) fn key_identifier_for_layout(
    identifier: &str,
    layout_index: usize,
) -> Option<&'static str> {
    if (layout_index != 0 && identifier.starts_with("num_"))
        || (layout_index == 2
            && matches!(
                identifier,
                "quote"
                    | "insert"
                    | "home"
                    | "page_up"
                    | "delete"
                    | "end"
                    | "page_down"
                    | "arrow_up"
                    | "arrow_down"
                    | "arrow_left"
                    | "arrow_right"
            ))
    {
        return None;
    }
    let identifier = match (layout_index, identifier) {
        (2, "left_shift") => "shift_left",
        (2, "right_shift") => "shift_right",
        (2, "left_ctrl") => "ctrl_left",
        (2, "right_ctrl") => "ctrl_right",
        (2, "left_meta") => "win_left",
        (2, "right_meta") => "win_right",
        (2, "left_alt") => "alt_left",
        (2, "right_alt") => "alt_right",
        (0 | 1, "shift_left") => "left_shift",
        (0 | 1, "shift_right") => "right_shift",
        (0 | 1, "ctrl_left") => "left_ctrl",
        (0 | 1, "ctrl_right") => "right_ctrl",
        (0 | 1, "win_left") => "left_meta",
        (0 | 1, "win_right") => "right_meta",
        (0 | 1, "alt_left") => "left_alt",
        (0 | 1, "alt_right") => "right_alt",
        _ => identifier,
    };
    key_identifier(identifier)
}
