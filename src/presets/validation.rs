use super::model::{PresetFile, PresetKind};

use super::paths::preset_filename;
use crate::persistence::STORAGE_VERSION;
use uuid::Uuid;

pub(super) fn validate_preset(preset: &PresetFile, kind: PresetKind) -> Result<(), String> {
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
    if let Some(state) = &preset.keyboard
        && let Some(layouts) = &state.layout_bindings
        && (state.layout_index > 2 || layouts[state.layout_index] != preset.bindings)
    {
        return Err("The active keyboard layout does not match its playback bindings.".to_string());
    }
    preset_filename(&preset.name)?;
    Ok(())
}

pub(crate) fn validate_preset_name(name: &str) -> Result<(), String> {
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
