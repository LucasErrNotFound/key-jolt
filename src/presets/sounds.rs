use super::model::SoundReference;
use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

pub(super) fn copy_sound(
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

pub(super) fn reusable_sound_filename(source: &Path) -> Option<String> {
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

pub(super) fn valid_extension(source: &Path) -> String {
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

pub(super) fn new_sound_filename(extension: &str) -> String {
    format!("{}{}", Uuid::new_v4(), extension)
}

pub(super) fn files_equal(left: &Path, right: &Path) -> Result<bool, String> {
    let left_metadata = fs::metadata(left).map_err(|error| error.to_string())?;
    let right_metadata = fs::metadata(right).map_err(|error| error.to_string())?;
    if left_metadata.len() != right_metadata.len() {
        return Ok(false);
    }
    let left = fs::read(left).map_err(|error| error.to_string())?;
    let right = fs::read(right).map_err(|error| error.to_string())?;
    Ok(left == right)
}
