use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::view::home_view::{
    KeyboardPresetData, KeyboardSoundData, MousePresetData, MouseSoundData,
};

pub const STORAGE_VERSION: u32 = 1;
const PRESET_MANIFEST_NAME: &str = "preset.json";
const MAX_PACKAGE_FILES: usize = 512;
const MAX_PACKAGE_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetKind {
    Keyboard,
    Mouse,
}

impl PresetKind {
    pub fn directory(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    Sequential,
    Random,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundReference {
    pub file: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub sounds: Vec<SoundReference>,
    pub playback_mode: PlaybackMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardPresetState {
    pub layout_index: usize,
    pub sync_selections: bool,
    pub selected_keys: [Vec<String>; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetFile {
    pub version: u32,
    #[serde(default)]
    pub kind: Option<PresetKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<KeyboardPresetState>,
    pub id: String,
    pub name: String,
    pub summary: String,
    pub bindings: BTreeMap<String, Binding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedPreset {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub kind: PresetKind,
    pub keyboard: Option<KeyboardPresetData>,
    pub mouse: Option<MousePresetData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetWarning {
    pub message: String,
}

pub fn data_dir() -> Result<PathBuf, String> {
    ProjectDirs::from("com", "KeyJolt", "key-jolt")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| "Could not find the operating system app data directory.".to_string())
}

pub fn resolve_preset_id(requested: Option<&str>, is_user_preset: bool) -> String {
    if is_user_preset && let Some(id) = requested {
        return id.to_string();
    }
    Uuid::new_v4().to_string()
}

pub fn preset_package_path(root: &Path, kind: PresetKind, name: &str) -> Result<PathBuf, String> {
    Ok(root
        .join("presets")
        .join(kind.directory())
        .join(preset_filename(name)?))
}

pub fn runtime_bindings(
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

pub fn delete_preset(root: &Path, kind: PresetKind, id: &str) -> Result<(), String> {
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

pub fn save_keyboard_preset(
    root: &Path,
    id: &str,
    name: &str,
    summary: &str,
    data: &KeyboardPresetData,
) -> Result<PresetFile, String> {
    save_preset_package(
        root,
        PresetKind::Keyboard,
        id,
        name,
        summary,
        Some(keyboard_preset_state(data)),
        |directory| keyboard_bindings(data, directory),
    )
}

pub fn save_mouse_preset(
    root: &Path,
    id: &str,
    name: &str,
    summary: &str,
    data: &MousePresetData,
) -> Result<PresetFile, String> {
    save_preset_package(
        root,
        PresetKind::Mouse,
        id,
        name,
        summary,
        None,
        |directory| mouse_bindings(data, directory),
    )
}

pub fn load_preset(root: &Path, kind: PresetKind, id: &str) -> Result<LoadedPreset, String> {
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

pub fn import_preset(root: &Path, kind: PresetKind, source: &Path) -> Result<LoadedPreset, String> {
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

fn save_preset_package<F>(
    root: &Path,
    kind: PresetKind,
    id: &str,
    name: &str,
    summary: &str,
    keyboard: Option<KeyboardPresetState>,
    build_bindings: F,
) -> Result<PresetFile, String>
where
    F: FnOnce(&Path) -> Result<BTreeMap<String, Binding>, String>,
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
        let bindings = build_bindings(&stage)?;
        let preset = save_preset_file(&stage, kind, id, name, summary, keyboard, bindings)?;
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

fn save_preset_file(
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

fn keyboard_preset_state(data: &KeyboardPresetData) -> KeyboardPresetState {
    KeyboardPresetState {
        layout_index: data.layout_index.min(2),
        sync_selections: data.sync_selections,
        selected_keys: data
            .selected_keys
            .clone()
            .map(|keys| keys.into_iter().map(str::to_string).collect::<Vec<_>>()),
    }
}

fn keyboard_bindings(
    data: &KeyboardPresetData,
    directory: &Path,
) -> Result<BTreeMap<String, Binding>, String> {
    let sounds = data
        .files
        .iter()
        .filter(|sound| sound.selected)
        .map(|sound| copy_sound(&sound.path, &sound.name, directory))
        .collect::<Result<Vec<_>, _>>()?;
    let mut bindings = BTreeMap::new();
    for key in data.selected_keys.iter().flatten() {
        bindings.insert(
            runtime_key_identifier(key).to_string(),
            Binding {
                sounds: sounds.clone(),
                playback_mode: if data.random_playback {
                    PlaybackMode::Random
                } else {
                    PlaybackMode::Sequential
                },
            },
        );
    }
    Ok(bindings)
}

fn mouse_bindings(
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

fn copy_sound(
    source: &Path,
    display_name: &str,
    directory: &Path,
) -> Result<SoundReference, String> {
    if !source.is_file() {
        return Err(format!("Audio file is missing: {}", source.display()));
    }

    let sounds_directory = directory.join("sounds");
    fs::create_dir_all(&sounds_directory).map_err(|error| error.to_string())?;
    let extension = valid_extension(source);
    let mut filename =
        reusable_sound_filename(source).unwrap_or_else(|| new_sound_filename(&extension));
    let mut destination = sounds_directory.join(&filename);

    if destination.exists() && !files_equal(source, &destination)? {
        filename = new_sound_filename(&extension);
        destination = sounds_directory.join(&filename);
    }
    if !destination.exists() {
        fs::copy(source, &destination).map_err(|error| error.to_string())?;
    }

    Ok(SoundReference {
        file: PathBuf::from("sounds")
            .join(filename)
            .to_string_lossy()
            .replace('\\', "/"),
        name: display_name.to_string(),
        enabled: true,
    })
}

fn reusable_sound_filename(source: &Path) -> Option<String> {
    let filename = source.file_name()?.to_str()?;
    let stem = source.file_stem()?.to_str()?;
    let extension = source.extension().and_then(|extension| extension.to_str());
    if Uuid::parse_str(stem).is_ok()
        && extension.is_none_or(|extension| {
            !extension.is_empty()
                && extension
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
    {
        Some(filename.to_string())
    } else {
        None
    }
}

fn valid_extension(source: &Path) -> String {
    source
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| {
            !extension.is_empty()
                && extension
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default()
}

fn new_sound_filename(extension: &str) -> String {
    format!("{}{}", Uuid::new_v4(), extension)
}

fn files_equal(left: &Path, right: &Path) -> Result<bool, String> {
    let left_metadata = fs::metadata(left).map_err(|error| error.to_string())?;
    let right_metadata = fs::metadata(right).map_err(|error| error.to_string())?;
    if left_metadata.len() != right_metadata.len() {
        return Ok(false);
    }
    let left = fs::read(left).map_err(|error| error.to_string())?;
    let right = fs::read(right).map_err(|error| error.to_string())?;
    Ok(left == right)
}

pub fn load_presets(root: &Path) -> (Vec<LoadedPreset>, Vec<PresetWarning>) {
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

fn load_packaged_preset(path: &Path, kind: PresetKind) -> Result<LoadedPreset, String> {
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

fn load_legacy_preset(
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

fn load_preset_source(
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

fn validate_preset(preset: &PresetFile, kind: PresetKind) -> Result<(), String> {
    if preset.version != STORAGE_VERSION {
        return Err(format!("Unsupported preset version {}.", preset.version));
    }
    if Uuid::parse_str(&preset.id).is_err() {
        return Err("The preset has an invalid generated ID.".to_string());
    }
    if preset.kind.is_some_and(|preset_kind| preset_kind != kind) {
        return Err("The preset type does not match this section.".to_string());
    }
    if preset.bindings.is_empty() {
        return Err(format!("Preset '{}' has no bindings.", preset.name));
    }
    preset_filename(&preset.name)?;
    Ok(())
}

fn find_package_by_id(root: &Path, kind: PresetKind, id: &str) -> Result<Option<PathBuf>, String> {
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

fn read_package_manifest(path: &Path) -> Result<PresetFile, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    if archive.len() > MAX_PACKAGE_FILES {
        return Err("The preset archive contains too many files.".to_string());
    }
    let mut manifest = archive
        .by_name(PRESET_MANIFEST_NAME)
        .map_err(|_| "The preset archive is missing preset.json.".to_string())?;
    if manifest.size() > 1024 * 1024 {
        return Err("The preset manifest is too large.".to_string());
    }
    let mut bytes = Vec::new();
    manifest
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

fn write_package(directory: &Path, destination: &Path) -> Result<(), String> {
    let file = File::create(destination).map_err(|error| error.to_string())?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let manifest =
        fs::read(directory.join(PRESET_MANIFEST_NAME)).map_err(|error| error.to_string())?;
    writer
        .start_file(PRESET_MANIFEST_NAME, options)
        .map_err(|error| error.to_string())?;
    writer
        .write_all(&manifest)
        .map_err(|error| error.to_string())?;

    let sounds_directory = directory.join("sounds");
    if sounds_directory.is_dir() {
        for entry in fs::read_dir(&sounds_directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| "An audio file name is invalid.".to_string())?;
            let archive_name = format!("sounds/{filename}");
            writer
                .start_file(archive_name, options)
                .map_err(|error| error.to_string())?;
            let mut source = File::open(path).map_err(|error| error.to_string())?;
            std::io::copy(&mut source, &mut writer).map_err(|error| error.to_string())?;
        }
    }

    writer.finish().map_err(|error| error.to_string())?;
    Ok(())
}

fn extract_package(source: &Path, destination: &Path) -> Result<(), String> {
    let temporary = destination.with_file_name(format!(
        ".{}-{}",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("preset"),
        Uuid::new_v4()
    ));
    let result = (|| {
        if temporary.exists() {
            fs::remove_dir_all(&temporary).map_err(|error| error.to_string())?;
        }
        fs::create_dir_all(&temporary).map_err(|error| error.to_string())?;

        let file = File::open(source).map_err(|error| error.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
        if archive.len() > MAX_PACKAGE_FILES {
            return Err("The preset archive contains too many files.".to_string());
        }

        let total_uncompressed = (0..archive.len()).try_fold(0u64, |total, index| {
            let entry = archive.by_index(index).map_err(|error| error.to_string())?;
            total
                .checked_add(entry.size())
                .ok_or_else(|| "The preset archive is too large.".to_string())
        })?;
        if total_uncompressed > MAX_PACKAGE_UNCOMPRESSED_BYTES {
            return Err("The preset archive is too large.".to_string());
        }

        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
            if entry.is_dir() {
                continue;
            }
            let Some(relative) = safe_relative_path(entry.name()) else {
                return Err("The preset archive contains an unsafe path.".to_string());
            };
            if relative != Path::new(PRESET_MANIFEST_NAME) && !relative.starts_with("sounds") {
                continue;
            }
            let target = temporary.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            let mut output = File::create(target).map_err(|error| error.to_string())?;
            std::io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
        }

        if !temporary.join(PRESET_MANIFEST_NAME).is_file() {
            return Err("The preset archive is missing preset.json.".to_string());
        }
        if destination.exists() {
            fs::remove_dir_all(destination).map_err(|error| error.to_string())?;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::rename(&temporary, destination).map_err(|error| error.to_string())
    })();
    if result.is_err() && temporary.exists() {
        let _ = fs::remove_dir_all(temporary);
    }
    result
}

fn replace_file(temporary: &Path, target: &Path) -> Result<(), String> {
    if !target.exists() {
        return fs::rename(temporary, target).map_err(|error| error.to_string());
    }
    let backup = target.with_file_name(format!(".{}.bak", Uuid::new_v4()));
    fs::rename(target, &backup).map_err(|error| error.to_string())?;
    match fs::rename(temporary, target) {
        Ok(()) => {
            let _ = fs::remove_file(backup);
            Ok(())
        }
        Err(error) => {
            let _ = fs::rename(&backup, target);
            Err(error.to_string())
        }
    }
}

fn safe_relative_path(value: &str) -> Option<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() {
        return None;
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => return None,
        }
    }
    (!safe.as_os_str().is_empty()).then_some(safe)
}

pub fn validate_preset_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter a name for this preset.".to_string());
    }
    if name == "." || name == ".." || name.ends_with('.') || name.ends_with(' ') {
        return Err("The preset name cannot end with a dot or space.".to_string());
    }
    if name
        .chars()
        .any(|character| character.is_control() || "<>:\"/\\|?*".contains(character))
    {
        return Err(
            "The preset name contains a character that cannot be used in a file name.".to_string(),
        );
    }

    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .and_then(|value| value.parse::<u8>().ok())
            .is_some_and(|value| (1..=9).contains(&value))
        || stem
            .strip_prefix("LPT")
            .and_then(|value| value.parse::<u8>().ok())
            .is_some_and(|value| (1..=9).contains(&value));
    if reserved {
        return Err("The preset name is reserved by Windows.".to_string());
    }
    Ok(())
}

fn preset_filename(name: &str) -> Result<String, String> {
    validate_preset_name(name)?;
    Ok(format!("{}.zip", name.trim()))
}

fn is_zip_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

fn legacy_directory(root: &Path, kind: PresetKind, id: &str) -> PathBuf {
    root.join("presets").join(kind.directory()).join(id)
}

fn legacy_manifest_path(directory: &Path) -> Result<Option<PathBuf>, String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    Ok(entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        }))
}

fn cache_directory(kind: PresetKind, id: &str) -> PathBuf {
    std::env::temp_dir()
        .join("KeyJolt")
        .join("preset-cache")
        .join(kind.directory())
        .join(id)
}

fn remove_cache(kind: PresetKind, id: &str) {
    let directory = cache_directory(kind, id);
    if directory.exists() {
        let _ = fs::remove_dir_all(directory);
    }
}

fn staging_directory(purpose: &str) -> PathBuf {
    std::env::temp_dir()
        .join("KeyJolt")
        .join("staging")
        .join(format!("{purpose}-{}", Uuid::new_v4()))
}

fn loaded_preset(
    preset: PresetFile,
    kind: PresetKind,
    directory: &Path,
    warnings: &mut Vec<PresetWarning>,
) -> LoadedPreset {
    let mut available = BTreeMap::<String, PathBuf>::new();
    for (identifier, binding) in &preset.bindings {
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
        for binding in preset.bindings.values() {
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
                    selected: true,
                })
            })
            .collect();
        KeyboardPresetData {
            selected_keys,
            files,
            random_playback: preset
                .bindings
                .values()
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

fn file_size(path: &Path) -> Option<gpui_kit::SharedString> {
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

fn key_identifier(identifier: &str) -> Option<&'static str> {
    const KEYS: &[&str] = &[
        "esc",
        "f1",
        "f2",
        "f3",
        "f4",
        "f5",
        "f6",
        "f7",
        "f8",
        "f9",
        "f10",
        "f11",
        "f12",
        "print_screen",
        "grave",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "0",
        "minus",
        "equal",
        "backspace",
        "tab",
        "q",
        "w",
        "e",
        "r",
        "t",
        "y",
        "u",
        "i",
        "o",
        "p",
        "left_bracket",
        "right_bracket",
        "backslash",
        "caps_lock",
        "a",
        "s",
        "d",
        "f",
        "g",
        "h",
        "j",
        "k",
        "l",
        "semicolon",
        "quote",
        "enter",
        "shift_left",
        "z",
        "x",
        "c",
        "v",
        "b",
        "n",
        "m",
        "comma",
        "period",
        "slash",
        "shift_right",
        "ctrl_left",
        "win_left",
        "alt_left",
        "space",
        "alt_right",
        "win_right",
        "menu",
        "ctrl_right",
        "left_shift",
        "right_shift",
        "left_ctrl",
        "left_meta",
        "left_alt",
        "right_alt",
        "right_meta",
        "right_ctrl",
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
        "num_lock",
        "num_divide",
        "num_multiply",
        "num_minus",
        "num_7",
        "num_8",
        "num_9",
        "num_plus_1",
        "num_4",
        "num_5",
        "num_6",
        "num_plus_2",
        "num_1",
        "num_2",
        "num_3",
        "num_enter_1",
        "num_0",
        "num_decimal",
        "num_enter_2",
    ];
    KEYS.iter().copied().find(|key| *key == identifier)
}

fn runtime_key_identifier(identifier: &str) -> &str {
    match identifier {
        "left_shift" => "shift_left",
        "right_shift" => "shift_right",
        "left_ctrl" => "ctrl_left",
        "right_ctrl" => "ctrl_right",
        "left_meta" => "win_left",
        "right_meta" => "win_right",
        "left_alt" => "alt_left",
        "right_alt" => "alt_right",
        "num_plus_2" => "num_plus_1",
        "num_enter_2" => "num_enter_1",
        identifier => identifier,
    }
}

fn canonical_key_identifier(identifier: &str) -> &str {
    match identifier {
        "shift_left" => "left_shift",
        "shift_right" => "right_shift",
        "ctrl_left" => "left_ctrl",
        "ctrl_right" => "right_ctrl",
        "win_left" => "left_meta",
        "win_right" => "right_meta",
        "alt_left" => "left_alt",
        "alt_right" => "right_alt",
        "num_plus_2" => "num_plus_1",
        "num_enter_2" => "num_enter_1",
        identifier => identifier,
    }
}

fn keyboard_mapped_key_count(bindings: &BTreeMap<String, Binding>) -> usize {
    bindings
        .keys()
        .filter(|identifier| key_identifier(identifier).is_some())
        .map(|identifier| canonical_key_identifier(identifier).to_string())
        .collect::<BTreeSet<_>>()
        .len()
}

fn restore_keyboard_selection(preset: &PresetFile) -> ([Vec<&'static str>; 3], usize, bool) {
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

fn infer_legacy_keyboard_layout(bindings: &BTreeMap<String, Binding>) -> usize {
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

fn key_identifier_for_layout(identifier: &str, layout_index: usize) -> Option<&'static str> {
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

fn filename_slug(name: &str) -> String {
    let slug = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if slug.is_empty() {
        "preset".to_string()
    } else {
        slug
    }
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Storage path has no parent directory.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = parent.join(format!(".{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        let mut file = File::create(&temporary).map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&temporary, path).map_err(|error| error.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let package =
            preset_package_path(&source_root, PresetKind::Keyboard, "Shared Preset").unwrap();
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
}
