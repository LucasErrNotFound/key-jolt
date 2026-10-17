use super::archive::{extract_package, read_package_manifest, replace_file};
use super::model::{LoadedPreset, PresetFile, PresetKind, PresetWarning};
use super::paths::safe_relative_path;
use super::validation::validate_preset;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;
use zip::ZipArchive;

const KEYBOARD_ID: &str = "c772e030-794c-49b2-a4cc-758d81795df4";
const MOUSE_ID: &str = "df4dce3d-b060-4db7-89b9-f2fd603f5824";
const KEYBOARD_PACKAGE: &[u8] = include_bytes!("../../assets/presets/BoxJades.zip");
const MOUSE_PACKAGE: &[u8] = include_bytes!("../../assets/presets/DefaultMouseClick.zip");
static CACHE_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn builtin_preset_id(kind: PresetKind) -> &'static str {
    match kind {
        PresetKind::Keyboard => KEYBOARD_ID,
        PresetKind::Mouse => MOUSE_ID,
    }
}

pub(crate) fn is_builtin_preset(kind: PresetKind, id: &str) -> bool {
    id == builtin_preset_id(kind)
}

pub(super) fn is_reserved_id(id: &str) -> bool {
    id == KEYBOARD_ID || id == MOUSE_ID
}

pub(crate) fn startup_preset_id(
    kind: PresetKind,
    saved: Option<&str>,
    presets: &[LoadedPreset],
) -> Option<String> {
    let available = |id: &str| {
        presets
            .iter()
            .any(|preset| preset.kind == kind && preset.id == id)
    };
    saved
        .filter(|id| available(id))
        .or_else(|| Some(builtin_preset_id(kind)).filter(|id| available(id)))
        .map(str::to_string)
}

pub(super) fn load_all(root: &Path) -> (Vec<LoadedPreset>, Vec<PresetWarning>) {
    let mut presets = Vec::new();
    let mut warnings = Vec::new();
    for kind in [PresetKind::Keyboard, PresetKind::Mouse] {
        match super::repository::load_preset(root, kind, builtin_preset_id(kind)) {
            Ok(preset) => presets.push(preset),
            Err(error) => warnings.push(PresetWarning {
                message: format!(
                    "Could not load the built-in {} preset: {error}",
                    kind.directory()
                ),
            }),
        }
    }
    (presets, warnings)
}

pub(super) fn load_source(
    root: &Path,
    kind: PresetKind,
    id: &str,
) -> Result<Option<(PresetFile, PathBuf)>, String> {
    if !is_builtin_preset(kind, id) {
        return Ok(None);
    }
    if root.as_os_str().is_empty() {
        return Err("The app data directory is unavailable.".to_string());
    }
    let _guard = CACHE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let bytes = match kind {
        PresetKind::Keyboard => KEYBOARD_PACKAGE,
        PresetKind::Mouse => MOUSE_PACKAGE,
    };
    let fingerprint = bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    let base = root
        .join("bundled-presets")
        .join(format!("{id}-{fingerprint:016x}"));
    fs::create_dir_all(&base).map_err(|error| error.to_string())?;
    let package = base.join("preset.zip");
    if !file_matches(&package, bytes) {
        let temporary = base.join(format!(".{}.zip.tmp", Uuid::new_v4()));
        let result = fs::write(&temporary, bytes)
            .map_err(|error| error.to_string())
            .and_then(|()| replace_file(&temporary, &package));
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
    }
    let preset = registered_manifest(read_package_manifest(&package)?, kind)?;
    let directory = base.join("contents");
    if !cache_matches(bytes, &directory)? {
        extract_package(&package, &directory)?;
    }
    Ok(Some((preset, directory)))
}

pub(super) fn registered_manifest(
    mut preset: PresetFile,
    kind: PresetKind,
) -> Result<PresetFile, String> {
    validate_preset(&preset, kind)?;
    if preset.kind != Some(kind) {
        return Err("The bundled preset type does not match its registered channel.".to_string());
    }
    preset.id = builtin_preset_id(kind).to_string();
    Ok(preset)
}

fn file_matches(path: &Path, bytes: &[u8]) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        metadata.is_file()
            && metadata.len() == bytes.len() as u64
            && fs::read(path).is_ok_and(|stored| stored == bytes)
    })
}

fn cache_matches(bytes: &[u8], directory: &Path) -> Result<bool, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let relative = safe_relative_path(entry.name())
            .ok_or_else(|| "The bundled preset contains an unsafe path.".to_string())?;
        let path = directory.join(relative);
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            return Ok(false);
        };
        if !metadata.is_file() || metadata.len() != entry.size() {
            return Ok(false);
        }
        let Ok(mut stored) = File::open(path) else {
            return Ok(false);
        };
        let mut expected = [0_u8; 8192];
        let mut actual = [0_u8; 8192];
        loop {
            let count = entry
                .read(&mut expected)
                .map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            if stored.read_exact(&mut actual[..count]).is_err()
                || expected[..count] != actual[..count]
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
