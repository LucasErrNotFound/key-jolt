mod archive;
mod bindings;
mod builtin;
mod key_identifier;
mod model;
mod paths;
mod repository;
mod restoration;
mod sounds;
mod validation;

pub(crate) use bindings::runtime_bindings;
pub(crate) use builtin::{builtin_preset_id, is_builtin_preset, startup_preset_id};
pub(crate) use key_identifier::canonical_key_identifier;
pub(crate) use model::{
    KeyboardPresetData, KeyboardSoundData, LoadedPreset, MousePresetData, MouseSoundData,
    PlaybackMode, PresetData, PresetKind,
};
pub(crate) use repository::{
    delete_preset, import_preset, load_preset, load_presets, resolve_preset_id,
    save_keyboard_preset, save_mouse_preset,
};
pub(crate) use validation::validate_preset_name;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod builtin_tests;
