use super::model::PresetKind;
use super::validation::validate_preset_name;
use std::fs;
use std::path::{Component, Path, PathBuf};

use uuid::Uuid;

pub(crate) fn preset_package_path(
    root: &Path,
    kind: PresetKind,
    name: &str,
) -> Result<PathBuf, String> {
    Ok(root
        .join("presets")
        .join(kind.directory())
        .join(preset_filename(name)?))
}

pub(super) fn preset_filename(name: &str) -> Result<String, String> {
    validate_preset_name(name)?;
    Ok(format!("{}.zip", name.trim()))
}

pub(super) fn is_zip_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

pub(super) fn legacy_directory(root: &Path, kind: PresetKind, id: &str) -> PathBuf {
    root.join("presets").join(kind.directory()).join(id)
}

pub(super) fn legacy_manifest_path(directory: &Path) -> Result<Option<PathBuf>, String> {
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

pub(super) fn cache_directory(kind: PresetKind, id: &str) -> PathBuf {
    std::env::temp_dir()
        .join("KeyJolt")
        .join("preset-cache")
        .join(kind.directory())
        .join(id)
}

pub(super) fn remove_cache(kind: PresetKind, id: &str) {
    let directory = cache_directory(kind, id);
    if directory.exists() {
        let _ = fs::remove_dir_all(directory);
    }
}

pub(super) fn staging_directory(purpose: &str) -> PathBuf {
    std::env::temp_dir()
        .join("KeyJolt")
        .join("staging")
        .join(format!("{purpose}-{}", Uuid::new_v4()))
}

pub(super) fn safe_relative_path(value: &str) -> Option<PathBuf> {
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
