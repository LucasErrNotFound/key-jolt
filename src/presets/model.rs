use gpui_kit::SharedString;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) enum PresetData {
    Keyboard(KeyboardPresetData),
    Mouse(MousePresetData),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct KeyboardPresetData {
    pub selected_keys: [Vec<&'static str>; 3],
    pub files: Vec<KeyboardSoundData>,
    pub random_playback: bool,
    pub layout_index: usize,
    pub sync_selections: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct KeyboardSoundData {
    pub path: PathBuf,
    pub name: SharedString,
    pub size: Option<SharedString>,
    pub assigned_keys: [Vec<String>; 3],
    pub independent_assigned_keys: Option<[Vec<String>; 3]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MousePresetData {
    pub files: Vec<MouseSoundData>,
    pub random_playback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MouseSoundData {
    pub path: PathBuf,
    pub name: SharedString,
    pub size: Option<SharedString>,
    pub assigned_buttons: [bool; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PresetKind {
    Keyboard,
    Mouse,
}

impl PresetKind {
    pub(crate) fn directory(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlaybackMode {
    Sequential,
    Random,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SoundReference {
    pub file: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Binding {
    pub sounds: Vec<SoundReference>,
    pub playback_mode: PlaybackMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct KeyboardPresetState {
    pub layout_index: usize,
    pub sync_selections: bool,
    pub selected_keys: [Vec<String>; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_bindings: Option<[BTreeMap<String, Binding>; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub independent_layout_bindings: Option<[BTreeMap<String, Binding>; 3]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PresetFile {
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
pub(crate) struct LoadedPreset {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub kind: PresetKind,
    pub keyboard: Option<KeyboardPresetData>,
    pub mouse: Option<MousePresetData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PresetWarning {
    pub message: String,
}
