use super::archive::{
    PRESET_MANIFEST_NAME, extract_package, read_package_manifest, replace_file, write_package,
};

use super::bindings::{keyboard_bindings, keyboard_preset_state, mouse_bindings};

use super::model::{
    Binding, KeyboardPresetData, KeyboardPresetState, LoadedPreset, MousePresetData, PresetFile,
    PresetKind, PresetWarning,
};

use super::paths::{
    cache_directory, is_zip_file, legacy_directory, legacy_manifest_path, preset_package_path,
    remove_cache, staging_directory,
};

use super::restoration::loaded_preset;
use super::validation::validate_preset;
use crate::persistence::{STORAGE_VERSION, write_json_atomic};

use std::collections::{BTreeMap, BTreeSet};

use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

pub(crate) fn resolve_preset_id(requested: Option<&str>, is_user_preset: bool) -> String {
    if is_user_preset && let Some(id) = requested {
        return id.to_string();
    }
    Uuid::new_v4().to_string()
}

pub(crate) fn delete_preset(root: &Path, kind: PresetKind, id: &str) -> Result<(), String> {
    if root.as_os_str().is_empty() || Uuid::parse_str(id).is_err() {
        return Err("The selected preset cannot be deleted.".to_string());
    }

    if let Some(package) = find_package_by_id(root, kind, id)? {
        fs::remove_file(package).map_err(|error| error.to_string())?;
        remove_cache(kind, id);
        let legacy = legacy_directory(root, kind, id);
        if legacy.exists() {
            let _ = fs::remove_dir_all(legacy);
        }
        return Ok(());
    }

    let directory = legacy_directory(root, kind, id);
    let metadata = fs::symlink_metadata(&directory).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("The selected preset folder is invalid.".to_string());
    }
    let base = root.join("presets").join(kind.directory());
    let base = base.canonicalize().map_err(|error| error.to_string())?;
    let directory = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !directory.starts_with(base) {
        return Err("The selected preset folder is outside preset storage.".to_string());
    }
    fs::remove_dir_all(directory).map_err(|error| error.to_string())?;
    remove_cache(kind, id);
    Ok(())
}

pub(crate) fn save_keyboard_preset(
    root: &Path,
    id: &str,
    name: &str,
    summary: &str,
    data: &KeyboardPresetData,
) -> Result<PresetFile, String> {
    save_preset_package(root, PresetKind::Keyboard, id, name, summary, |directory| {
        let mappings = keyboard_bindings(data, directory)?;
        let bindings = mappings.layouts[data.layout_index.min(2)].clone();
        Ok((Some(keyboard_preset_state(data, mappings)), bindings))
    })
}

pub(crate) fn save_mouse_preset(
    root: &Path,
    id: &str,
    name: &str,
    summary: &str,
    data: &MousePresetData,
) -> Result<PresetFile, String> {
    save_preset_package(root, PresetKind::Mouse, id, name, summary, |directory| {
        Ok((None, mouse_bindings(data, directory)?))
    })
}

pub(crate) fn load_preset(root: &Path, kind: PresetKind, id: &str) -> Result<LoadedPreset, String> {
    let Some((preset, directory)) = load_preset_source(root, kind, id)? else {
        return Err("The preset could not be found.".to_string());
    };
    validate_preset(&preset, kind)?;
    let mut warnings = Vec::new();
    let loaded = loaded_preset(preset, kind, &directory, &mut warnings);
    if warnings.is_empty() {
        Ok(loaded)
    } else {
        Err(warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect::<Vec<_>>()
            .join(" "))
    }
}

pub(crate) fn import_preset(
    root: &Path,
    kind: PresetKind,
    source: &Path,
) -> Result<LoadedPreset, String> {
    if root.as_os_str().is_empty() {
        return Err("The app data directory is unavailable.".to_string());
    }
    if !source.is_file() {
        return Err("The selected preset archive could not be found.".to_string());
    }

    let preset = read_package_manifest(source)?;
    validate_preset(&preset, kind)?;
    if preset.kind != Some(kind) {
        return Err(format!(
            "This is a {} preset, not a {} preset.",
            preset.kind.map(PresetKind::directory).unwrap_or("legacy"),
            kind.directory()
        ));
    }

    let temporary = staging_directory("import");
    let result = (|| {
        extract_package(source, &temporary)?;
        let mut warnings = Vec::new();
        let loaded = loaded_preset(preset.clone(), kind, &temporary, &mut warnings);
        if !warnings.is_empty() {
            return Err(warnings
                .into_iter()
                .map(|warning| warning.message)
                .collect::<Vec<_>>()
                .join(" "));
        }

        let id = Uuid::new_v4().to_string();
        match kind {
            PresetKind::Keyboard => {
                let data = loaded
                    .keyboard
                    .as_ref()
                    .ok_or_else(|| "The keyboard preset data is missing.".to_string())?;
                save_keyboard_preset(root, &id, &preset.name, &preset.summary, data)?;
            }
            PresetKind::Mouse => {
                let data = loaded
                    .mouse
                    .as_ref()
                    .ok_or_else(|| "The mouse preset data is missing.".to_string())?;
                save_mouse_preset(root, &id, &preset.name, &preset.summary, data)?;
            }
        }
        load_preset(root, kind, &id)
    })();
    let _ = fs::remove_dir_all(temporary);
    result
}

pub(super) fn save_preset_package<F>(
    root: &Path,
    kind: PresetKind,
    id: &str,
    name: &str,
    summary: &str,
    build_bindings: F,
) -> Result<PresetFile, String>
where
    F: FnOnce(&Path) -> Result<(Option<KeyboardPresetState>, BTreeMap<String, Binding>), String>,
{
    if root.as_os_str().is_empty() {
        return Err("The app data directory is unavailable.".to_string());
    }
    if Uuid::parse_str(id).is_err() {
        return Err("The preset ID is invalid.".to_string());
    }

    let kind_directory = root.join("presets").join(kind.directory());
    fs::create_dir_all(&kind_directory).map_err(|error| error.to_string())?;
    let target = preset_package_path(root, kind, name)?;
    let existing = find_package_by_id(root, kind, id)?;

    if target.exists() && existing.as_deref() != Some(target.as_path()) {
        let target_preset = read_package_manifest(&target)?;
        if target_preset.id != id {
            return Err(format!("A preset named \"{name}\" already exists."));
        }
    }

    let stage = staging_directory("save");
    let archive_temporary = kind_directory.join(format!(".{}.zip.tmp", Uuid::new_v4()));
    let result = (|| {
        fs::create_dir_all(stage.join("sounds")).map_err(|error| error.to_string())?;
        let (keyboard, bindings) = build_bindings(&stage)?;
        let preset = save_preset_file(&stage, kind, id, name, summary, keyboard, bindings)?;
        validate_preset(&preset, kind)?;
        write_package(&stage, &archive_temporary)?;
        replace_file(&archive_temporary, &target)?;

        if let Some(existing) = existing.as_ref()
            && existing != &target
        {
            let _ = fs::remove_file(existing);
        }

        let legacy = legacy_directory(root, kind, id);
        if legacy.exists() {
            let _ = fs::remove_dir_all(legacy);
        }
        Ok(preset)
    })();

    let _ = fs::remove_dir_all(stage);
    if archive_temporary.exists() {
        let _ = fs::remove_file(archive_temporary);
    }
    result
}

pub(super) fn save_preset_file(
    directory: &Path,
    kind: PresetKind,
    id: &str,
    name: &str,
    summary: &str,
    keyboard: Option<KeyboardPresetState>,
    bindings: BTreeMap<String, Binding>,
) -> Result<PresetFile, String> {
    let preset = PresetFile {
        version: STORAGE_VERSION,
        kind: Some(kind),
        keyboard,
        id: id.to_string(),
        name: name.to_string(),
        summary: summary.to_string(),
        bindings,
    };
    write_json_atomic(&directory.join(PRESET_MANIFEST_NAME), &preset)?;
    Ok(preset)
}

pub(crate) fn load_presets(root: &Path) -> (Vec<LoadedPreset>, Vec<PresetWarning>) {
    let mut presets = Vec::new();
    let mut warnings = Vec::new();
    if root.as_os_str().is_empty() {
        return (presets, warnings);
    }

    for kind in [PresetKind::Keyboard, PresetKind::Mouse] {
        let kind_directory = root.join("presets").join(kind.directory());
        let entries = match fs::read_dir(&kind_directory) {
            Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                warnings.push(PresetWarning {
                    message: format!("Could not read {} presets: {error}", kind.directory()),
                });
                continue;
            }
        };

        let mut seen_ids = BTreeSet::new();
        for entry in entries.iter().filter(|entry| is_zip_file(&entry.path())) {
            match load_packaged_preset(&entry.path(), kind) {
                Ok(preset) => {
                    seen_ids.insert(preset.id.clone());
                    presets.push(preset);
                }
                Err(error) => warnings.push(PresetWarning {
                    message: format!("Skipped preset archive {}: {error}", entry.path().display()),
                }),
            }
        }

        for entry in entries.iter().filter(|entry| entry.path().is_dir()) {
            match load_legacy_preset(&entry.path(), kind, &mut warnings) {
                Ok(Some(preset)) if !seen_ids.contains(&preset.id) => {
                    presets.push(preset);
                }
                Ok(_) => {}
                Err(error) => warnings.push(PresetWarning {
                    message: format!("Skipped legacy preset {}: {error}", entry.path().display()),
                }),
            }
        }
    }
    (presets, warnings)
}

pub(super) fn load_packaged_preset(path: &Path, kind: PresetKind) -> Result<LoadedPreset, String> {
    let preset = read_package_manifest(path)?;
    validate_preset(&preset, kind)?;
    if preset.kind != Some(kind) {
        return Err("The archive contains a different preset type.".to_string());
    }
    let directory = cache_directory(kind, &preset.id);
    extract_package(path, &directory)?;
    let mut warnings = Vec::new();
    let loaded = loaded_preset(preset, kind, &directory, &mut warnings);
    if warnings.is_empty() {
        Ok(loaded)
    } else {
        Err(warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect::<Vec<_>>()
            .join(" "))
    }
}

pub(super) fn load_legacy_preset(
    directory: &Path,
    kind: PresetKind,
    warnings: &mut Vec<PresetWarning>,
) -> Result<Option<LoadedPreset>, String> {
    let Some(file_path) = legacy_manifest_path(directory)? else {
        warnings.push(PresetWarning {
            message: format!("Skipped empty preset folder: {}", directory.display()),
        });
        return Ok(None);
    };
    let mut preset: PresetFile =
        serde_json::from_slice(&fs::read(&file_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    if preset.kind.is_none() {
        preset.kind = Some(kind);
    }
    validate_preset(&preset, kind)?;
    let directory_id = directory.file_name().and_then(|name| name.to_str());
    if directory_id != Some(preset.id.as_str()) {
        return Err("The preset folder ID does not match the preset file.".to_string());
    }
    Ok(Some(loaded_preset(preset, kind, directory, warnings)))
}

pub(super) fn load_preset_source(
    root: &Path,
    kind: PresetKind,
    id: &str,
) -> Result<Option<(PresetFile, PathBuf)>, String> {
    if let Some(package) = find_package_by_id(root, kind, id)? {
        let preset = read_package_manifest(&package)?;
        validate_preset(&preset, kind)?;
        let directory = cache_directory(kind, id);
        extract_package(&package, &directory)?;
        return Ok(Some((preset, directory)));
    }

    let directory = legacy_directory(root, kind, id);
    if !directory.is_dir() {
        return Ok(None);
    }
    let Some(file_path) = legacy_manifest_path(&directory)? else {
        return Err("The active preset file could not be found.".to_string());
    };
    let mut preset: PresetFile =
        serde_json::from_slice(&fs::read(file_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    if preset.kind.is_none() {
        preset.kind = Some(kind);
    }
    validate_preset(&preset, kind)?;
    Ok(Some((preset, directory)))
}

pub(super) fn find_package_by_id(
    root: &Path,
    kind: PresetKind,
    id: &str,
) -> Result<Option<PathBuf>, String> {
    let directory = root.join("presets").join(kind.directory());
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !is_zip_file(&path) {
            continue;
        }
        let Ok(preset) = read_package_manifest(&path) else {
            continue;
        };
        if preset.id == id && preset.kind.is_none_or(|preset_kind| preset_kind == kind) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
