use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::persistence::{STORAGE_VERSION, data_dir, write_json_atomic};
use crate::settings::appearance::{AppearanceMode, DEFAULT_LIGHT_THEME_ID};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub version: u32,
    pub keyboard_volume: f32,
    pub keyboard_muted: bool,
    pub mouse_volume: f32,
    pub mouse_muted: bool,
    pub app_enabled: bool,
    #[serde(default)]
    pub run_at_startup: bool,
    pub active_keyboard_preset_id: Option<String>,
    pub active_mouse_preset_id: Option<String>,
    #[serde(default)]
    pub appearance_mode: AppearanceMode,
    #[serde(default = "default_appearance_theme_id")]
    pub appearance_theme_id: String,
}

fn default_appearance_theme_id() -> String {
    DEFAULT_LIGHT_THEME_ID.to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: STORAGE_VERSION,
            keyboard_volume: 50.0,
            keyboard_muted: false,
            mouse_volume: 50.0,
            mouse_muted: false,
            app_enabled: true,
            run_at_startup: false,
            active_keyboard_preset_id: Some("cherry-mx-blue".to_string()),
            active_mouse_preset_id: Some("mouse-sounds".to_string()),
            appearance_mode: AppearanceMode::default(),
            appearance_theme_id: default_appearance_theme_id(),
        }
    }
}

impl AppSettings {
    pub fn clamp(&mut self) {
        self.keyboard_volume = self.keyboard_volume.clamp(0.0, 200.0);
        self.mouse_volume = self.mouse_volume.clamp(0.0, 200.0);
        self.version = STORAGE_VERSION;
    }

    pub fn effective_keyboard_volume(&self) -> f32 {
        if self.keyboard_muted {
            0.0
        } else {
            self.keyboard_volume / 100.0
        }
    }

    pub fn effective_mouse_volume(&self) -> f32 {
        if self.mouse_muted {
            0.0
        } else {
            self.mouse_volume / 100.0
        }
    }
}

pub fn settings_path(root: &Path) -> PathBuf {
    root.join("settings.json")
}

pub fn load() -> (PathBuf, AppSettings, Option<String>) {
    let root = match data_dir() {
        Ok(root) => root,
        Err(error) => return (PathBuf::new(), AppSettings::default(), Some(error)),
    };
    let (settings, warning) = load_or_default(&root);
    (root, settings, warning)
}

pub fn load_from(root: &Path) -> Result<AppSettings, String> {
    let bytes = std::fs::read(settings_path(root)).map_err(|error| error.to_string())?;
    let mut settings: AppSettings =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if settings.version != STORAGE_VERSION {
        return Err(format!(
            "Unsupported settings version {}.",
            settings.version
        ));
    }
    settings.clamp();
    Ok(settings)
}

pub fn load_or_default(root: &Path) -> (AppSettings, Option<String>) {
    if !settings_path(root).exists() {
        return (AppSettings::default(), None);
    }
    match load_from(root) {
        Ok(settings) => (settings, None),
        Err(error) => (AppSettings::default(), Some(error)),
    }
}

pub fn save_to(root: &Path, settings: &AppSettings) -> Result<(), String> {
    if root.as_os_str().is_empty() {
        return Err("The app data directory is unavailable.".to_string());
    }
    let mut settings = settings.clone();
    settings.clamp();
    write_json_atomic(&settings_path(root), &settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_clamp_values() {
        let mut settings = AppSettings {
            keyboard_volume: 250.0,
            mouse_volume: -5.0,
            ..AppSettings::default()
        };
        settings.clamp();
        let json = serde_json::to_string(&settings).unwrap();
        let decoded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, settings);
        assert_eq!(decoded.keyboard_volume, 200.0);
        assert_eq!(decoded.mouse_volume, 0.0);
    }

    #[test]
    fn corrupt_settings_are_rejected_for_default_fallback() {
        let root = std::env::temp_dir().join(format!("key-jolt-settings-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(settings_path(&root), b"invalid").unwrap();
        assert!(load_from(&root).is_err());
        assert_eq!(load_or_default(&root).0, AppSettings::default());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn atomic_settings_write_replaces_an_existing_file() {
        let root =
            std::env::temp_dir().join(format!("key-jolt-settings-save-{}", uuid::Uuid::new_v4()));
        let mut settings = AppSettings::default();
        save_to(&root, &settings).unwrap();
        settings.keyboard_volume = 120.0;
        settings.run_at_startup = true;
        save_to(&root, &settings).unwrap();
        let loaded = load_from(&root).unwrap();
        assert_eq!(loaded.keyboard_volume, 120.0);
        assert!(loaded.run_at_startup);
        settings.run_at_startup = false;
        save_to(&root, &settings).unwrap();
        assert!(!load_from(&root).unwrap().run_at_startup);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn older_settings_without_appearance_load_with_defaults() {
        let json = r#"{"version":1,"keyboard_volume":50.0,"keyboard_muted":false,"mouse_volume":50.0,"mouse_muted":false,"app_enabled":true,"active_keyboard_preset_id":null,"active_mouse_preset_id":null}"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();

        assert!(!settings.run_at_startup);
        assert_eq!(settings.appearance_mode, AppearanceMode::Light);
        assert_eq!(settings.appearance_theme_id, DEFAULT_LIGHT_THEME_ID);
    }
}
