use super::*;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const MOUSE_PACKAGE: &[u8] = include_bytes!("../../assets/presets/DefaultMouseClick.zip");

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("keyjolt-builtin-test-{}", Uuid::new_v4())))
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn bundles_load_as_two_real_presets_with_original_mappings() {
    let root = TestRoot::new();
    let (presets, warnings) = load_presets(&root.0);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(presets.len(), 2);
    let keyboard = presets
        .iter()
        .find(|preset| preset.kind == PresetKind::Keyboard)
        .unwrap();
    assert_eq!(keyboard.name, "BoxJades");
    assert!(is_builtin_preset(keyboard.kind, &keyboard.id));
    let data = keyboard.keyboard.as_ref().unwrap();
    assert_eq!(data.layout_index, 0);
    assert!(data.sync_selections);
    assert!(data.random_playback);
    assert_eq!(data.files.len(), 4);
    assert!(data.files.iter().all(|file| file.path.is_file()));
    let names = data
        .files
        .iter()
        .map(|file| file.name.as_ref())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        ["Key1.ogg", "Key2.ogg", "Key3.ogg", "Key4.ogg"]
            .into_iter()
            .collect()
    );
    for (layout, expected) in [102, 85, 74].into_iter().enumerate() {
        let assigned = data
            .files
            .iter()
            .flat_map(|file| file.assigned_keys[layout].iter())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(assigned.len(), expected);
    }
    assert!(data.files.iter().all(|file| {
        file.independent_assigned_keys
            .as_ref()
            .is_some_and(|layouts| {
                layouts[0] == file.assigned_keys[0]
                    && layouts[1].is_empty()
                    && layouts[2].is_empty()
            })
    }));
    let keyboard_bindings = runtime_bindings(&root.0, PresetKind::Keyboard, &keyboard.id).unwrap();
    assert_eq!(keyboard_bindings.len(), 102);
    assert!(
        keyboard_bindings
            .values()
            .all(|(mode, sounds)| *mode == PlaybackMode::Random
                && sounds.len() == 4
                && sounds.iter().all(|path| path.is_file()))
    );
    let mouse = presets
        .iter()
        .find(|preset| preset.kind == PresetKind::Mouse)
        .unwrap();
    assert_eq!(mouse.name, "DefaultMouseClick");
    let data = mouse.mouse.as_ref().unwrap();
    assert_eq!(data.files.len(), 1);
    assert_eq!(data.files[0].assigned_buttons, [true; 3]);
    assert_eq!(data.files[0].name.as_ref(), "Default_Mouse_Click.ogg");
    assert!(!data.random_playback);
    let mouse_bindings = runtime_bindings(&root.0, PresetKind::Mouse, &mouse.id).unwrap();
    assert_eq!(mouse_bindings.len(), 3);
    assert!(
        mouse_bindings
            .values()
            .all(|(mode, sounds)| *mode == PlaybackMode::Sequential
                && sounds.len() == 1
                && sounds[0].is_file())
    );
}

#[test]
fn cache_is_reused_and_missing_or_changed_audio_is_repaired() {
    let root = TestRoot::new();
    let id = builtin_preset_id(PresetKind::Mouse);
    let loaded = load_preset(&root.0, PresetKind::Mouse, id).unwrap();
    let path = loaded.mouse.unwrap().files[0].path.clone();
    let original = fs::read(&path).unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    load_preset(&root.0, PresetKind::Mouse, id).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    let mut corrupted = original.clone();
    corrupted[0] ^= 0xff;
    fs::write(&path, corrupted).unwrap();
    load_preset(&root.0, PresetKind::Mouse, id).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(&path).unwrap();
    let bindings = runtime_bindings(&root.0, PresetKind::Mouse, id).unwrap();
    assert_eq!(bindings.len(), 3);
    assert_eq!(fs::read(path).unwrap(), original);
}

#[test]
fn corrupted_cached_archive_is_replaced_from_embedded_bytes() {
    let root = TestRoot::new();
    let id = builtin_preset_id(PresetKind::Mouse);
    let loaded = load_preset(&root.0, PresetKind::Mouse, id).unwrap();
    let audio = &loaded.mouse.as_ref().unwrap().files[0].path;
    let contents = audio.parent().unwrap().parent().unwrap();
    let package = contents.parent().unwrap().join("preset.zip");
    fs::write(&package, b"corrupted").unwrap();
    assert!(load_preset(&root.0, PresetKind::Mouse, id).is_ok());
    assert_eq!(fs::read(package).unwrap(), MOUSE_PACKAGE);
}

#[test]
fn builtin_ids_cannot_be_deleted_or_overwritten() {
    let root = TestRoot::new();
    for kind in [PresetKind::Keyboard, PresetKind::Mouse] {
        let id = builtin_preset_id(kind);
        let original = load_preset(&root.0, kind, id).unwrap();
        assert!(delete_preset(&root.0, kind, id).is_err());
        let result = match kind {
            PresetKind::Keyboard => save_keyboard_preset(
                &root.0,
                id,
                "Changed",
                "",
                original.keyboard.as_ref().unwrap(),
            ),
            PresetKind::Mouse => {
                save_mouse_preset(&root.0, id, "Changed", "", original.mouse.as_ref().unwrap())
            }
        };
        assert!(result.is_err());
        assert_eq!(load_preset(&root.0, kind, id).unwrap(), original);
        assert_ne!(resolve_preset_id(Some(id), true), id);
        assert_ne!(resolve_preset_id(Some(id), false), id);
    }
}

#[test]
fn editing_builtin_saves_independent_custom_copy() {
    let root = TestRoot::new();
    for kind in [PresetKind::Keyboard, PresetKind::Mouse] {
        let builtin_id = builtin_preset_id(kind);
        let original = load_preset(&root.0, kind, builtin_id).unwrap();
        let custom_id = resolve_preset_id(Some(builtin_id), false);
        let result = match kind {
            PresetKind::Keyboard => save_keyboard_preset(
                &root.0,
                &custom_id,
                "Custom Keyboard",
                "",
                original.keyboard.as_ref().unwrap(),
            ),
            PresetKind::Mouse => save_mouse_preset(
                &root.0,
                &custom_id,
                "Custom Mouse",
                "",
                original.mouse.as_ref().unwrap(),
            ),
        };
        result.unwrap();
        let custom = load_preset(&root.0, kind, &custom_id).unwrap();
        assert!(!is_builtin_preset(kind, &custom.id));
        assert_eq!(load_preset(&root.0, kind, builtin_id).unwrap(), original);
        assert_eq!(resolve_preset_id(Some(&custom_id), true), custom_id);
        let before = runtime_bindings(&root.0, kind, builtin_id).unwrap();
        let after = runtime_bindings(&root.0, kind, &custom_id).unwrap();
        assert_eq!(
            before.keys().collect::<Vec<_>>(),
            after.keys().collect::<Vec<_>>()
        );
        for (key, (mode, paths)) in before {
            let (other_mode, other_paths) = &after[&key];
            assert_eq!(mode, *other_mode);
            assert_eq!(paths.len(), other_paths.len());
            let mut expected = paths
                .iter()
                .map(|path| fs::read(path).unwrap())
                .collect::<Vec<_>>();
            let mut actual = other_paths
                .iter()
                .map(|path| fs::read(path).unwrap())
                .collect::<Vec<_>>();
            if mode == PlaybackMode::Random {
                expected.sort();
                actual.sort();
            }
            assert!(
                expected == actual,
                "The custom copy changed the sound pool for {key}"
            );
        }
        assert_eq!(
            startup_preset_id(kind, Some(&custom_id), &load_presets(&root.0).0),
            Some(custom_id)
        );
    }
}

#[test]
fn startup_defaults_only_replace_missing_or_invalid_choices() {
    let root = TestRoot::new();
    let (presets, _) = load_presets(&root.0);
    for kind in [PresetKind::Keyboard, PresetKind::Mouse] {
        let id = builtin_preset_id(kind);
        assert_eq!(startup_preset_id(kind, None, &presets).as_deref(), Some(id));
        assert_eq!(
            startup_preset_id(kind, Some("missing"), &presets).as_deref(),
            Some(id)
        );
        assert_eq!(
            startup_preset_id(kind, Some(id), &presets).as_deref(),
            Some(id)
        );
        assert_eq!(startup_preset_id(kind, Some(id), &[]), None);
    }
    assert_eq!(
        startup_preset_id(
            PresetKind::Keyboard,
            Some(builtin_preset_id(PresetKind::Mouse)),
            &presets
        )
        .as_deref(),
        Some(builtin_preset_id(PresetKind::Keyboard))
    );
}

#[test]
fn user_archives_with_reserved_ids_do_not_shadow_or_duplicate_builtins() {
    let root = TestRoot::new();
    let folder = root.0.join("presets/mouse");
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join("Shadow.zip");
    let staging = root.0.join("shadow-source");
    let source = root.0.join("source.zip");
    fs::write(&source, MOUSE_PACKAGE).unwrap();
    super::archive::extract_package(&source, &staging).unwrap();
    let mut manifest = super::archive::read_package_manifest(&source).unwrap();
    manifest.id = builtin_preset_id(PresetKind::Mouse).to_string();
    fs::write(
        staging.join("preset.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    super::archive::write_package(&staging, &file).unwrap();
    let original = fs::read(&file).unwrap();
    let (presets, warnings) = load_presets(&root.0);
    assert_eq!(presets.len(), 2);
    assert_eq!(warnings.len(), 1);
    assert_eq!(fs::read(file).unwrap(), original);
}

#[test]
fn missing_data_directory_does_not_create_relative_cache() {
    assert!(load_presets(std::path::Path::new("")).0.is_empty());
    assert!(
        load_preset(
            std::path::Path::new(""),
            PresetKind::Mouse,
            builtin_preset_id(PresetKind::Mouse)
        )
        .is_err()
    );
}

#[test]
fn new_settings_select_both_builtins_and_saved_preferences_survive() {
    use crate::settings::store::{AppSettings, load_from, save_to};
    let root = TestRoot::new();
    let defaults = AppSettings::default();
    assert_eq!(
        defaults.active_keyboard_preset_id.as_deref(),
        Some(builtin_preset_id(PresetKind::Keyboard))
    );
    assert_eq!(
        defaults.active_mouse_preset_id.as_deref(),
        Some(builtin_preset_id(PresetKind::Mouse))
    );
    let keyboard = load_preset(
        &root.0,
        PresetKind::Keyboard,
        builtin_preset_id(PresetKind::Keyboard),
    )
    .unwrap();
    let custom_id = Uuid::new_v4().to_string();
    save_keyboard_preset(
        &root.0,
        &custom_id,
        "Existing Choice",
        "",
        keyboard.keyboard.as_ref().unwrap(),
    )
    .unwrap();
    let saved = AppSettings {
        active_keyboard_preset_id: Some(custom_id.clone()),
        keyboard_volume: 37.0,
        mouse_volume: 162.0,
        keyboard_muted: true,
        mouse_muted: true,
        app_enabled: false,
        run_at_startup: true,
        ..defaults
    };
    save_to(&root.0, &saved).unwrap();
    let mut restored = load_from(&root.0).unwrap();
    let loaded = load_presets(&root.0).0;
    restored.active_keyboard_preset_id = startup_preset_id(
        PresetKind::Keyboard,
        restored.active_keyboard_preset_id.as_deref(),
        &loaded,
    );
    restored.active_mouse_preset_id = startup_preset_id(
        PresetKind::Mouse,
        restored.active_mouse_preset_id.as_deref(),
        &loaded,
    );
    assert_eq!(restored, saved);
}

#[test]
fn exported_uuids_are_normalized_only_for_registered_bundles() {
    let root = TestRoot::new();
    for kind in [PresetKind::Keyboard, PresetKind::Mouse] {
        let id = builtin_preset_id(kind);
        let (original, directory) = builtin::load_source(&root.0, kind, id).unwrap().unwrap();
        let cached: super::model::PresetFile =
            serde_json::from_slice(&fs::read(directory.join("preset.json")).unwrap()).unwrap();
        assert_eq!(original.id, id);
        assert_eq!(original.bindings, cached.bindings);
        assert_eq!(original.keyboard, cached.keyboard);
        for _ in 0..2 {
            let mut exported = cached.clone();
            exported.id = Uuid::new_v4().to_string();
            let expected = exported.clone();
            let normalized = builtin::registered_manifest(exported, kind).unwrap();
            assert_eq!(normalized.id, id);
            assert_eq!(normalized.kind, expected.kind);
            assert_eq!(normalized.name, expected.name);
            assert_eq!(normalized.summary, expected.summary);
            assert_eq!(normalized.bindings, expected.bindings);
            assert_eq!(normalized.keyboard, expected.keyboard);
        }
    }
}

#[test]
fn registering_bundles_still_rejects_invalid_manifest_metadata() {
    let root = TestRoot::new();
    let kind = PresetKind::Keyboard;
    let (original, _) = builtin::load_source(&root.0, kind, builtin_preset_id(kind))
        .unwrap()
        .unwrap();
    let mut wrong_kind = original.clone();
    wrong_kind.kind = Some(PresetKind::Mouse);
    assert!(builtin::registered_manifest(wrong_kind, kind).is_err());
    let mut missing_kind = original.clone();
    missing_kind.kind = None;
    assert!(builtin::registered_manifest(missing_kind, kind).is_err());
    let mut invalid_id = original.clone();
    invalid_id.id = "invalid-id".to_string();
    assert!(builtin::registered_manifest(invalid_id, kind).is_err());
    let mut wrong_version = original;
    wrong_version.version += 1;
    assert!(builtin::registered_manifest(wrong_version, kind).is_err());
}
