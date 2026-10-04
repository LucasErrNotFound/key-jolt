use super::archive::PRESET_MANIFEST_NAME;
use super::model::{Binding, PresetFile, SoundReference};
use super::paths::{legacy_directory, preset_package_path, remove_cache};
use super::*;
use crate::persistence::STORAGE_VERSION;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::Path;
use uuid::Uuid;
use zip::ZipArchive;

#[test]
fn archive_traversal_is_rejected_before_committing_extraction() {
    use super::archive::extract_package;
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let root = std::env::temp_dir().join(format!("key-jolt-unsafe-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let source = root.join("unsafe.zip");
    let file = File::create(&source).unwrap();
    let mut writer = ZipWriter::new(file);
    writer
        .start_file("../outside.wav", SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"audio").unwrap();
    writer
        .start_file("preset.json", SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"{}").unwrap();
    writer.finish().unwrap();

    let destination = root.join("extracted");
    assert!(extract_package(&source, &destination).is_err());
    assert!(!destination.exists());
    assert!(!root.join("outside.wav").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preset_names_reject_reserved_or_unsafe_file_names() {
    for name in ["", "../escape", "A/B", "CON", "COM1", "LPT9.txt", "ends."] {
        assert!(validate_preset_name(name).is_err(), "{name}");
    }
    assert!(validate_preset_name("My Keyboard Preset").is_ok());
}

fn archive_entries(path: &Path) -> Vec<String> {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    (0..archive.len())
        .map(|index| archive.by_index(index).unwrap().name().to_string())
        .collect()
}

#[test]
fn preset_json_round_trip_includes_kind() {
    let preset = PresetFile {
        version: STORAGE_VERSION,
        kind: Some(PresetKind::Mouse),
        keyboard: None,
        id: Uuid::new_v4().to_string(),
        name: "Preset".to_string(),
        summary: "Summary".to_string(),
        bindings: BTreeMap::from([(
            "left".to_string(),
            Binding {
                sounds: vec![SoundReference {
                    file: "sounds/audio-id.wav".to_string(),
                    name: "click.wav".to_string(),
                    enabled: true,
                }],
                playback_mode: PlaybackMode::Random,
            },
        )]),
    };
    let json = serde_json::to_string(&preset).unwrap();
    let decoded: PresetFile = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, preset);
}

#[test]
fn edited_builtin_gets_a_new_id() {
    let builtin_id = "cherry-mx-blue";
    let saved_id = resolve_preset_id(Some(builtin_id), false);
    assert_ne!(saved_id, builtin_id);
}

#[test]
fn saving_keyboard_creates_named_zip_and_runtime_cache() {
    let root = std::env::temp_dir().join(format!("key-jolt-preset-{}", Uuid::new_v4()));
    let source = root.join("source.wav");
    fs::create_dir_all(&root).unwrap();
    fs::write(&source, b"audio bytes").unwrap();
    let data = KeyboardPresetData {
        selected_keys: [vec![], vec![], vec!["esc"]],
        files: vec![KeyboardSoundData {
            path: source.clone(),
            name: "source.wav".into(),
            size: None,
            selected: true,
        }],
        random_playback: false,
        layout_index: 2,
        sync_selections: false,
    };
    let id = Uuid::new_v4().to_string();
    let saved = save_keyboard_preset(&root, &id, "Test Preset", "summary", &data).unwrap();
    assert_eq!(saved.kind, Some(PresetKind::Keyboard));
    assert_eq!(saved.keyboard.as_ref().unwrap().layout_index, 2);
    assert!(!saved.keyboard.as_ref().unwrap().sync_selections);
    assert_eq!(
        saved.keyboard.as_ref().unwrap().selected_keys[2],
        vec!["esc"]
    );
    let package = preset_package_path(&root, PresetKind::Keyboard, "Test Preset").unwrap();
    assert!(package.is_file());
    assert_eq!(package.file_name().unwrap(), "Test Preset.zip");
    assert!(!legacy_directory(&root, PresetKind::Keyboard, &id).exists());
    let runtime = runtime_bindings(&root, PresetKind::Keyboard, &id).unwrap();
    assert_eq!(runtime["esc"].0, PlaybackMode::Sequential);
    assert_eq!(runtime["esc"].1.len(), 1);
    assert!(runtime["esc"].1[0].is_file());
    let loaded = load_preset(&root, PresetKind::Keyboard, &id).unwrap();
    assert_eq!(loaded.summary, "Custom · 1 key mapped");
    let keyboard = loaded.keyboard.unwrap();
    assert_eq!(keyboard.layout_index, 2);
    assert!(!keyboard.sync_selections);
    assert!(keyboard.selected_keys[0].is_empty());
    assert!(keyboard.selected_keys[1].is_empty());
    assert_eq!(keyboard.selected_keys[2], vec!["esc"]);
    let entries = archive_entries(&package);
    assert!(entries.iter().any(|entry| entry == PRESET_MANIFEST_NAME));
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.starts_with("sounds/"))
            .count(),
        1
    );
    let _ = fs::remove_dir_all(root);
    remove_cache(PresetKind::Keyboard, &id);
}

#[test]
fn editing_same_preset_overwrites_zip_without_duplicate_audio() {
    let root = std::env::temp_dir().join(format!("key-jolt-edit-{}", Uuid::new_v4()));
    let source = root.join("mouse.wav");
    fs::create_dir_all(&root).unwrap();
    fs::write(&source, b"mouse audio").unwrap();
    let id = Uuid::new_v4().to_string();
    let data = MousePresetData {
        files: vec![MouseSoundData {
            path: source,
            name: "mouse.wav".into(),
            size: None,
            assigned_buttons: [true, false, false],
        }],
        random_playback: false,
    };
    save_mouse_preset(&root, &id, "Mouse", "summary", &data).unwrap();
    let loaded = load_preset(&root, PresetKind::Mouse, &id).unwrap();
    let data = loaded.mouse.unwrap();
    let first = save_mouse_preset(&root, &id, "Mouse", "summary", &data).unwrap();
    let first_file = first.bindings["left"].sounds[0].file.clone();
    let second = save_mouse_preset(&root, &id, "Mouse", "summary", &data).unwrap();
    let second_file = second.bindings["left"].sounds[0].file.clone();
    assert_eq!(first_file, second_file);
    let package = preset_package_path(&root, PresetKind::Mouse, "Mouse").unwrap();
    let entries = archive_entries(&package);
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.starts_with("sounds/"))
            .count(),
        1
    );
    let _ = fs::remove_dir_all(root);
    remove_cache(PresetKind::Mouse, &id);
}

#[test]
fn import_creates_new_id_from_keyboard_package() {
    let source_root = std::env::temp_dir().join(format!("key-jolt-export-{}", Uuid::new_v4()));
    let target_root = std::env::temp_dir().join(format!("key-jolt-import-{}", Uuid::new_v4()));
    let audio = source_root.join("sound.wav");
    fs::create_dir_all(&source_root).unwrap();
    fs::write(&audio, b"audio").unwrap();
    let original_id = Uuid::new_v4().to_string();
    let data = KeyboardPresetData {
        selected_keys: [vec![], vec![], vec!["esc"]],
        files: vec![KeyboardSoundData {
            path: audio,
            name: "sound.wav".into(),
            size: None,
            selected: true,
        }],
        random_playback: false,
        layout_index: 2,
        sync_selections: false,
    };
    save_keyboard_preset(
        &source_root,
        &original_id,
        "Shared Preset",
        "summary",
        &data,
    )
    .unwrap();
    let package = preset_package_path(&source_root, PresetKind::Keyboard, "Shared Preset").unwrap();
    let imported = import_preset(&target_root, PresetKind::Keyboard, &package).unwrap();
    assert_ne!(imported.id, original_id);
    assert_eq!(imported.name, "Shared Preset");
    assert!(
        preset_package_path(&target_root, PresetKind::Keyboard, "Shared Preset")
            .unwrap()
            .is_file()
    );
    let _ = fs::remove_dir_all(source_root);
    let _ = fs::remove_dir_all(target_root);
    remove_cache(PresetKind::Keyboard, &original_id);
    remove_cache(PresetKind::Keyboard, &imported.id);
}

#[test]
fn keyboard_bindings_use_runtime_modifier_identifiers_and_print_screen() {
    let root = std::env::temp_dir().join(format!("key-jolt-runtime-{}", Uuid::new_v4()));
    let source = root.join("sound.wav");
    fs::create_dir_all(&root).unwrap();
    fs::write(&source, b"audio").unwrap();
    let id = Uuid::new_v4().to_string();
    let data = KeyboardPresetData {
        selected_keys: [
            vec![
                "left_ctrl",
                "right_meta",
                "left_alt",
                "right_shift",
                "print_screen",
            ],
            vec![],
            vec![],
        ],
        files: vec![KeyboardSoundData {
            path: source,
            name: "sound.wav".into(),
            size: None,
            selected: true,
        }],
        random_playback: false,
        layout_index: 0,
        sync_selections: false,
    };
    let saved = save_keyboard_preset(&root, &id, "Runtime Keys", "summary", &data).unwrap();
    for key in [
        "ctrl_left",
        "win_right",
        "alt_left",
        "shift_right",
        "print_screen",
    ] {
        assert!(saved.bindings.contains_key(key));
    }
    for key in ["left_ctrl", "right_meta", "left_alt", "right_shift"] {
        assert!(!saved.bindings.contains_key(key));
    }
    let runtime = runtime_bindings(&root, PresetKind::Keyboard, &id).unwrap();
    for key in [
        "ctrl_left",
        "win_right",
        "alt_left",
        "shift_right",
        "print_screen",
    ] {
        assert!(runtime.contains_key(key));
    }
    let _ = fs::remove_dir_all(root);
    remove_cache(PresetKind::Keyboard, &id);
}

#[test]
fn legacy_folder_still_loads() {
    let root = std::env::temp_dir().join(format!("key-jolt-legacy-{}", Uuid::new_v4()));
    let id = Uuid::new_v4().to_string();
    let directory = legacy_directory(&root, PresetKind::Keyboard, &id);
    fs::create_dir_all(directory.join("sounds")).unwrap();
    fs::write(directory.join("sounds/sound.wav"), b"audio").unwrap();
    let preset = PresetFile {
        version: STORAGE_VERSION,
        kind: None,
        keyboard: None,
        id: id.clone(),
        name: "Legacy".to_string(),
        summary: "summary".to_string(),
        bindings: BTreeMap::from([(
            "esc".to_string(),
            Binding {
                sounds: vec![SoundReference {
                    file: "sounds/sound.wav".to_string(),
                    name: "sound.wav".to_string(),
                    enabled: true,
                }],
                playback_mode: PlaybackMode::Sequential,
            },
        )]),
    };
    fs::write(
        directory.join(format!("{id}-legacy.json")),
        serde_json::to_vec(&preset).unwrap(),
    )
    .unwrap();
    let (presets, warnings) = load_presets(&root);
    assert_eq!(presets.len(), 1);
    assert!(warnings.is_empty());
    assert_eq!(presets[0].id, id);
    assert_eq!(presets[0].summary, "Custom · 1 key mapped");
    let keyboard = presets[0].keyboard.as_ref().unwrap();
    assert_eq!(keyboard.layout_index, 2);
    assert!(!keyboard.sync_selections);
    assert!(keyboard.selected_keys[0].is_empty());
    assert!(keyboard.selected_keys[1].is_empty());
    assert_eq!(keyboard.selected_keys[2], vec!["esc"]);
    let _ = fs::remove_dir_all(root);
}
