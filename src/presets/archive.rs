use super::model::PresetFile;
use super::paths::safe_relative_path;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use zip::write::SimpleFileOptions;

pub(super) const PRESET_MANIFEST_NAME: &str = "preset.json";

const MAX_PACKAGE_FILES: usize = 512;

const MAX_PACKAGE_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;

pub(super) fn read_package_manifest(path: &Path) -> Result<PresetFile, String> {
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

pub(super) fn write_package(directory: &Path, destination: &Path) -> Result<(), String> {
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

pub(super) fn extract_package(source: &Path, destination: &Path) -> Result<(), String> {
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

pub(super) fn replace_file(temporary: &Path, target: &Path) -> Result<(), String> {
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
