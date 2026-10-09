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
            assigned_keys: layout_keys(2, ["esc"]),
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
            assigned_keys: layout_keys(2, ["esc"]),
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
            assigned_keys: layout_keys(
                0,
                [
                    "left_ctrl",
                    "right_meta",
                    "left_alt",
                    "right_shift",
                    "print_screen",
                ],
            ),
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
fn independent_keyboard_sounds_survive_save_reopen_import_and_resave() {
    let root = std::env::temp_dir().join(format!("key-jolt-independent-{}", Uuid::new_v4()));
    let imported_root = root.join("imported");
    fs::create_dir_all(&root).unwrap();
    let a = root.join("letters.wav");
    let b = root.join("space.wav");
    fs::write(&a, b"letters audio").unwrap();
    fs::write(&b, b"space audio").unwrap();
    let letters = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];
    let data = KeyboardPresetData {
        selected_keys: [Vec::new(), Vec::new(), vec!["space"]],
        files: vec![
            KeyboardSoundData {
                path: a,
                name: "letters.wav".into(),
                size: None,
                assigned_keys: layout_keys(2, letters),
            },
            KeyboardSoundData {
                path: b,
                name: "space.wav".into(),
                size: None,
                assigned_keys: layout_keys(2, ["space"]),
            },
        ],
        random_playback: false,
        layout_index: 2,
        sync_selections: false,
    };
    let id = Uuid::new_v4().to_string();
    let saved = save_keyboard_preset(&root, &id, "Separate Keys", "summary", &data).unwrap();
    assert_eq!(saved.bindings.len(), 27);
    for letter in letters {
        assert_eq!(saved.bindings[letter].sounds.len(), 1);
        assert_eq!(saved.bindings[letter].sounds[0].name, "letters.wav");
    }
    assert_eq!(saved.bindings["space"].sounds.len(), 1);
    assert_eq!(saved.bindings["space"].sounds[0].name, "space.wav");

    let loaded = load_preset(&root, PresetKind::Keyboard, &id).unwrap();
    assert_eq!(loaded.summary, "Custom · 27 keys mapped");
    let restored = loaded.keyboard.unwrap();
    assert_eq!(restored.selected_keys[2], vec!["space"]);
    let letters_file = restored
        .files
        .iter()
        .find(|file| file.name.as_ref() == "letters.wav")
        .unwrap();
    let space_file = restored
        .files
        .iter()
        .find(|file| file.name.as_ref() == "space.wav")
        .unwrap();
    assert_eq!(letters_file.assigned_keys[2].len(), 26);
    assert!(
        !letters_file.assigned_keys[2]
            .iter()
            .any(|key| key == "space")
    );
    assert_eq!(space_file.assigned_keys[2], vec!["space"]);
    let resaved = save_keyboard_preset(&root, &id, "Separate Keys", "summary", &restored).unwrap();
    assert_eq!(resaved.bindings, saved.bindings);

    let package = preset_package_path(&root, PresetKind::Keyboard, "Separate Keys").unwrap();
    let imported = import_preset(&imported_root, PresetKind::Keyboard, &package).unwrap();
    let imported_data = imported.keyboard.unwrap();
    let imported_saved = save_keyboard_preset(
        &imported_root,
        &imported.id,
        "Separate Keys",
        "summary",
        &imported_data,
    )
    .unwrap();
    assert_eq!(imported_saved.bindings, saved.bindings);
    let runtime = runtime_bindings(&imported_root, PresetKind::Keyboard, &imported.id).unwrap();
    assert_eq!(fs::read(&runtime["a"].1[0]).unwrap(), b"letters audio");
    assert_eq!(fs::read(&runtime["space"].1[0]).unwrap(), b"space audio");
    let entries = archive_entries(&package);
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.starts_with("sounds/"))
            .count(),
        2
    );
    fs::remove_dir_all(root).unwrap();
    remove_cache(PresetKind::Keyboard, &id);
    remove_cache(PresetKind::Keyboard, &imported.id);
}

#[test]
fn clearing_keyboard_selection_does_not_remove_saved_bindings() {
    let root = std::env::temp_dir().join(format!("key-jolt-clear-selection-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let source = root.join("sound.wav");
    fs::write(&source, b"sound").unwrap();
    let data = KeyboardPresetData {
        selected_keys: [Vec::new(), Vec::new(), Vec::new()],
        files: vec![KeyboardSoundData {
            path: source,
            name: "sound.wav".into(),
            size: None,
            assigned_keys: layout_keys(2, ["a", "space"]),
        }],
        random_playback: false,
        layout_index: 2,
        sync_selections: false,
    };
    let id = Uuid::new_v4().to_string();
    let saved = save_keyboard_preset(&root, &id, "No Selection", "summary", &data).unwrap();
    assert_eq!(saved.bindings.len(), 2);
    let loaded = load_preset(&root, PresetKind::Keyboard, &id)
        .unwrap()
        .keyboard
        .unwrap();
    assert!(loaded.selected_keys.iter().all(Vec::is_empty));
    assert_eq!(loaded.files[0].assigned_keys[2], vec!["a", "space"]);
    fs::remove_dir_all(root).unwrap();
    remove_cache(PresetKind::Keyboard, &id);
}

#[test]
fn keyboard_sound_pools_and_modifier_aliases_stay_independent() {
    let root = std::env::temp_dir().join(format!("key-jolt-pools-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut files = Vec::new();
    for (name, keys) in [
        ("first.wav", vec!["a", "ctrl_left", "left_ctrl"]),
        ("second.wav", vec!["a"]),
        ("space.wav", vec!["space"]),
    ] {
        let path = root.join(name);
        fs::write(&path, name.as_bytes()).unwrap();
        files.push(KeyboardSoundData {
            path,
            name: name.into(),
            size: None,
            assigned_keys: layout_keys(2, keys),
        });
    }
    files.push(KeyboardSoundData {
        path: root.join("unassigned-missing.wav"),
        name: "unassigned-missing.wav".into(),
        size: None,
        assigned_keys: std::array::from_fn(|_| Vec::new()),
    });
    let data = KeyboardPresetData {
        selected_keys: [Vec::new(), Vec::new(), vec!["space"]],
        files,
        random_playback: true,
        layout_index: 2,
        sync_selections: true,
    };
    let id = Uuid::new_v4().to_string();
    let saved = save_keyboard_preset(&root, &id, "Sound Pools", "summary", &data).unwrap();
    assert_eq!(saved.bindings.len(), 3);
    assert_eq!(saved.bindings["a"].sounds.len(), 2);
    assert_eq!(saved.bindings["space"].sounds.len(), 1);
    assert_eq!(saved.bindings["ctrl_left"].sounds.len(), 1);
    let loaded = load_preset(&root, PresetKind::Keyboard, &id)
        .unwrap()
        .keyboard
        .unwrap();
    assert!(loaded.random_playback);
    assert_eq!(loaded.files.len(), 3);
    let resaved = save_keyboard_preset(&root, &id, "Sound Pools", "summary", &loaded).unwrap();
    for key in ["a", "space", "ctrl_left"] {
        let names = |binding: &Binding| {
            binding
                .sounds
                .iter()
                .map(|sound| sound.name.clone())
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(names(&resaved.bindings[key]), names(&saved.bindings[key]));
        assert_eq!(resaved.bindings[key].playback_mode, PlaybackMode::Random);
    }
    fs::remove_dir_all(root).unwrap();
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
    assert_eq!(keyboard.files[0].assigned_keys[2], vec!["esc"]);
    let _ = fs::remove_dir_all(root);
}

fn layout_keys(index: usize, keys: impl IntoIterator<Item = impl AsRef<str>>) -> [Vec<String>; 3] {
    let mut assignments = std::array::from_fn(|_| Vec::new());
    assignments[index] = keys
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .collect();
    assignments
}

#[test]
fn independent_layout_mappings_survive_reopen_import_and_active_layout_changes() {
    let root = std::env::temp_dir().join(format!("key-jolt-layouts-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut files = Vec::new();
    for (name, index) in [("full.wav", 0), ("tkl.wav", 1), ("compact.wav", 2)] {
        let path = root.join(name);
        fs::write(&path, name).unwrap();
        files.push(KeyboardSoundData {
            path,
            name: name.into(),
            size: None,
            assigned_keys: layout_keys(index, ["space", "a"]),
        });
    }
    let data = KeyboardPresetData {
        selected_keys: std::array::from_fn(|_| Vec::new()),
        files,
        random_playback: false,
        layout_index: 2,
        sync_selections: false,
    };
    let id = Uuid::new_v4().to_string();
    let saved = save_keyboard_preset(&root, &id, "Layout Mappings", "summary", &data).unwrap();
    assert_eq!(saved.bindings["space"].sounds[0].name, "compact.wav");
    let layouts = saved
        .keyboard
        .as_ref()
        .unwrap()
        .layout_bindings
        .as_ref()
        .unwrap();
    for (index, name) in ["full.wav", "tkl.wav", "compact.wav"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(layouts[index]["space"].sounds.len(), 1);
        assert_eq!(layouts[index]["space"].sounds[0].name, name);
    }
    let package = preset_package_path(&root, PresetKind::Keyboard, "Layout Mappings").unwrap();
    assert_eq!(
        archive_entries(&package)
            .iter()
            .filter(|entry| entry.starts_with("sounds/"))
            .count(),
        3
    );
    let imported_root = root.join("imported");
    let imported = import_preset(&imported_root, PresetKind::Keyboard, &package).unwrap();
    let mut restored = imported.keyboard.unwrap();
    for (index, name) in ["full.wav", "tkl.wav", "compact.wav"]
        .into_iter()
        .enumerate()
    {
        let sound = restored
            .files
            .iter()
            .find(|sound| sound.name.as_ref() == name)
            .unwrap();
        assert_eq!(sound.assigned_keys, layout_keys(index, ["a", "space"]));
    }
    restored.layout_index = 0;
    let updated = save_keyboard_preset(
        &imported_root,
        &imported.id,
        "Layout Mappings",
        "summary",
        &restored,
    )
    .unwrap();
    let runtime = runtime_bindings(&imported_root, PresetKind::Keyboard, &imported.id).unwrap();
    assert_eq!(fs::read(&runtime["space"].1[0]).unwrap(), b"full.wav");
    assert_eq!(
        updated.keyboard.as_ref().unwrap().layout_bindings,
        saved.keyboard.as_ref().unwrap().layout_bindings
    );
    let reopened = load_preset(&imported_root, PresetKind::Keyboard, &imported.id)
        .unwrap()
        .keyboard
        .unwrap();
    assert_eq!(reopened.layout_index, 0);
    assert_eq!(reopened.files.len(), 3);
    remove_cache(PresetKind::Keyboard, &id);
    remove_cache(PresetKind::Keyboard, &imported.id);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn older_keyboard_metadata_restores_only_the_saved_layout_when_sync_is_off() {
    let root = std::env::temp_dir().join(format!("key-jolt-old-layout-{}", Uuid::new_v4()));
    fs::create_dir_all(root.join("sounds")).unwrap();
    fs::write(root.join("sounds/test.wav"), b"old").unwrap();
    let make_preset = |sync| {
        serde_json::from_value::<PresetFile>(serde_json::json!({
        "version": STORAGE_VERSION, "kind": "keyboard", "id": Uuid::new_v4().to_string(), "name": "Older Layout", "summary": "summary",
        "keyboard": {"layout_index": 0, "sync_selections": sync, "selected_keys": [["a"], [], []]},
        "bindings": {"a": {"playback_mode": "sequential", "sounds": [{"file": "sounds/test.wav", "name": "test.wav", "enabled": true}]},
            "num_1": {"playback_mode": "sequential", "sounds": [{"file": "sounds/test.wav", "name": "test.wav", "enabled": true}]}}
    })).unwrap()
    };
    let mut warnings = Vec::new();
    let independent = super::restoration::loaded_preset(
        make_preset(false),
        PresetKind::Keyboard,
        &root,
        &mut warnings,
    )
    .keyboard
    .unwrap();
    assert_eq!(
        independent.files[0].assigned_keys,
        layout_keys(0, ["a", "num_1"])
    );
    let synced = super::restoration::loaded_preset(
        make_preset(true),
        PresetKind::Keyboard,
        &root,
        &mut warnings,
    )
    .keyboard
    .unwrap();
    assert_eq!(
        synced.files[0].assigned_keys,
        [vec!["a", "num_1"], vec!["a"], vec!["a"]]
            .map(|keys| keys.into_iter().map(str::to_string).collect::<Vec<_>>())
    );
    assert!(warnings.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inactive_layout_sound_references_receive_the_same_path_validation() {
    let root = std::env::temp_dir().join(format!("key-jolt-layout-path-{}", Uuid::new_v4()));
    fs::create_dir_all(root.join("sounds")).unwrap();
    let path = root.join("sounds/audio.wav");
    fs::write(&path, b"sound").unwrap();
    let data = KeyboardPresetData {
        selected_keys: std::array::from_fn(|_| Vec::new()),
        files: vec![KeyboardSoundData {
            path,
            name: "audio.wav".into(),
            size: None,
            assigned_keys: [vec!["a".into()], Vec::new(), vec!["a".into()]],
        }],
        random_playback: false,
        layout_index: 2,
        sync_selections: false,
    };
    let id = Uuid::new_v4().to_string();
    let mut saved = save_keyboard_preset(&root, &id, "Layout Paths", "summary", &data).unwrap();
    let archive = preset_package_path(&root, PresetKind::Keyboard, "Layout Paths").unwrap();
    let extracted = root.join("extracted");
    super::archive::extract_package(&archive, &extracted).unwrap();
    saved
        .keyboard
        .as_mut()
        .unwrap()
        .layout_bindings
        .as_mut()
        .unwrap()[0]
        .get_mut("a")
        .unwrap()
        .sounds[0]
        .file = "../outside.wav".into();
    let mut warnings = Vec::new();
    let _ = super::restoration::loaded_preset(
        saved.clone(),
        PresetKind::Keyboard,
        &extracted,
        &mut warnings,
    );
    assert!(!warnings.is_empty());
    saved.keyboard.as_mut().unwrap().layout_index = 1;
    assert!(super::validation::validate_preset(&saved, PresetKind::Keyboard).is_err());
    remove_cache(PresetKind::Keyboard, &id);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn saving_an_empty_active_layout_keeps_the_existing_preset_intact() {
    let root = std::env::temp_dir().join(format!("key-jolt-empty-layout-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("audio.wav");
    fs::write(&path, b"sound").unwrap();
    let mut data = KeyboardPresetData {
        selected_keys: std::array::from_fn(|_| Vec::new()),
        files: vec![KeyboardSoundData {
            path,
            name: "audio.wav".into(),
            size: None,
            assigned_keys: layout_keys(2, ["a"]),
        }],
        random_playback: false,
        layout_index: 2,
        sync_selections: false,
    };
    let id = Uuid::new_v4().to_string();
    save_keyboard_preset(&root, &id, "Keep Mappings", "summary", &data).unwrap();
    let package = preset_package_path(&root, PresetKind::Keyboard, "Keep Mappings").unwrap();
    let bytes = fs::read(&package).unwrap();
    data.layout_index = 0;
    assert!(save_keyboard_preset(&root, &id, "Keep Mappings", "summary", &data).is_err());
    assert_eq!(fs::read(&package).unwrap(), bytes);
    assert_eq!(
        load_preset(&root, PresetKind::Keyboard, &id)
            .unwrap()
            .keyboard
            .unwrap()
            .layout_index,
        2
    );
    remove_cache(PresetKind::Keyboard, &id);
    fs::remove_dir_all(root).unwrap();
}
