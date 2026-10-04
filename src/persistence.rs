use directories::ProjectDirs;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(crate) const STORAGE_VERSION: u32 = 1;

pub fn data_dir() -> Result<PathBuf, String> {
    ProjectDirs::from("com", "KeyJolt", "key-jolt")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| "Could not find the operating system app data directory.".to_string())
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
