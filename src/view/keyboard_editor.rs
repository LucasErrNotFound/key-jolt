use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::time::Duration;

use super::home_view::{KeyboardPresetData, KeyboardSoundData};
use gpui_kit::assets::IconName;
use gpui_kit::base::{Checkbox, CheckboxIndicator, CheckboxState};
use gpui_kit::component::WindowExt;
use gpui_kit::component::attachment::{
    Attachment, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentMedia,
    AttachmentStatus, AttachmentTitle,
};
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};
use gpui_kit::component::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::Label;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

#[derive(Clone, Debug)]
pub enum KeyboardEditorEvent {
    BackRequested,
    PreviewRequested {
        file_id: u64,
        path: PathBuf,
    },
    StopPreviewRequested {
        file_id: Option<u64>,
    },
    PlaybackStateChanged {
        state: PreviewState,
    },
    SaveRequested {
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        summary: SharedString,
        data: KeyboardPresetData,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewState {
    Stopped,
    Playing,
    Finished,
    Error(SharedString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveStatus {
    Idle,
    Saving,
    Failed(SharedString),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlaybackMode {
    Sequential,
    Random,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyboardLayout {
    FullSize,
    Tkl,
    Compact,
}

impl KeyboardLayout {
    const ALL: [Self; 3] = [Self::FullSize, Self::Tkl, Self::Compact];

    fn index(self) -> usize {
        match self {
            Self::FullSize => 0,
            Self::Tkl => 1,
            Self::Compact => 2,
        }
    }

    fn width(self) -> f32 {
        match self {
            Self::Compact => 520.0,
            Self::Tkl => 770.0,
            Self::FullSize => 950.0,
        }
    }

    fn key_ids(self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        let mut add_row = |row: &[KeySpec]| {
            keys.extend(
                row.iter()
                    .filter(|key| !key.id.starts_with("nav_up_spacer"))
                    .map(|key| key.id),
            );
        };

        match self {
            Self::Compact => {
                add_row(&ROW_1);
                add_row(&ROW_2);
                add_row(&ROW_3);
                add_row(&ROW_4);
                add_row(&ROW_5);
                add_row(&ROW_6);
            }
            Self::Tkl | Self::FullSize => {
                add_row(&TKL_FUNCTION_ROW);
                add_row(TKL_SYSTEM_ROW);
                add_row(TKL_MAIN_ROW_1);
                add_row(TKL_MAIN_ROW_2);
                add_row(TKL_MAIN_ROW_3);
                add_row(TKL_MAIN_ROW_4);
                add_row(TKL_MAIN_ROW_5);
                add_row(TKL_NAV_ROW_1);
                add_row(TKL_NAV_ROW_2);
                add_row(TKL_NAV_ROW_3);
                add_row(TKL_NAV_ROW_4);

                if self == Self::FullSize {
                    add_row(FULL_NUMPAD_ROW_1);
                    add_row(FULL_NUMPAD_ROW_2);
                    add_row(FULL_NUMPAD_ROW_3);
                    add_row(FULL_NUMPAD_ROW_4);
                    add_row(FULL_NUMPAD_ROW_5);
                }
            }
        }

        keys
    }
}

fn canonical_key_id(key_id: &str) -> &str {
    match key_id {
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
        key_id => key_id,
    }
}

struct SelectionGroup {
    id: &'static str,
    label: &'static str,
    keys: Vec<&'static str>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UploadState {
    Uploading,
    Success,
    Complete,
    Failed,
}

#[derive(Clone, Copy)]
struct KeySpec {
    id: &'static str,
    label: &'static str,
    width: f32,
}

#[derive(Clone)]
struct AudioFile {
    id: u64,
    path: PathBuf,
    name: SharedString,
    size: Option<SharedString>,
    selected: bool,
    state: UploadState,
    preview_state: PreviewState,
}

impl AudioFile {
    fn new(id: u64, path: PathBuf, name: impl Into<SharedString>) -> Self {
        Self {
            id,
            path,
            name: name.into(),
            size: None,
            selected: false,
            state: UploadState::Uploading,
            preview_state: PreviewState::Stopped,
        }
    }
}

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "oga", "opus", "m4a", "aac", "aiff", "aif", "wma", "webm", "ac3",
    "amr", "mid", "midi",
];

const ROW_1: [KeySpec; 14] = [
    KeySpec {
        id: "esc",
        label: "Esc",
        width: 32.0,
    },
    KeySpec {
        id: "f1",
        label: "F1",
        width: 32.0,
    },
    KeySpec {
        id: "f2",
        label: "F2",
        width: 32.0,
    },
    KeySpec {
        id: "f3",
        label: "F3",
        width: 32.0,
    },
    KeySpec {
        id: "f4",
        label: "F4",
        width: 32.0,
    },
    KeySpec {
        id: "f5",
        label: "F5",
        width: 32.0,
    },
    KeySpec {
        id: "f6",
        label: "F6",
        width: 32.0,
    },
    KeySpec {
        id: "f7",
        label: "F7",
        width: 32.0,
    },
    KeySpec {
        id: "f8",
        label: "F8",
        width: 32.0,
    },
    KeySpec {
        id: "f9",
        label: "F9",
        width: 32.0,
    },
    KeySpec {
        id: "f10",
        label: "F10",
        width: 32.0,
    },
    KeySpec {
        id: "f11",
        label: "F11",
        width: 32.0,
    },
    KeySpec {
        id: "f12",
        label: "F12",
        width: 32.0,
    },
    KeySpec {
        id: "print_screen",
        label: "PrtSc",
        width: 32.0,
    },
];

const ROW_2: [KeySpec; 14] = [
    KeySpec {
        id: "grave",
        label: "~",
        width: 32.0,
    },
    KeySpec {
        id: "1",
        label: "1",
        width: 32.0,
    },
    KeySpec {
        id: "2",
        label: "2",
        width: 32.0,
    },
    KeySpec {
        id: "3",
        label: "3",
        width: 32.0,
    },
    KeySpec {
        id: "4",
        label: "4",
        width: 32.0,
    },
    KeySpec {
        id: "5",
        label: "5",
        width: 32.0,
    },
    KeySpec {
        id: "6",
        label: "6",
        width: 32.0,
    },
    KeySpec {
        id: "7",
        label: "7",
        width: 32.0,
    },
    KeySpec {
        id: "8",
        label: "8",
        width: 32.0,
    },
    KeySpec {
        id: "9",
        label: "9",
        width: 32.0,
    },
    KeySpec {
        id: "0",
        label: "0",
        width: 32.0,
    },
    KeySpec {
        id: "minus",
        label: "-",
        width: 32.0,
    },
    KeySpec {
        id: "equal",
        label: "=",
        width: 32.0,
    },
    KeySpec {
        id: "backspace",
        label: "Backspace",
        width: 91.0,
    },
];

const ROW_3: [KeySpec; 14] = [
    KeySpec {
        id: "tab",
        label: "Tab",
        width: 65.0,
    },
    KeySpec {
        id: "q",
        label: "Q",
        width: 30.0,
    },
    KeySpec {
        id: "w",
        label: "W",
        width: 30.0,
    },
    KeySpec {
        id: "e",
        label: "E",
        width: 30.0,
    },
    KeySpec {
        id: "r",
        label: "R",
        width: 30.0,
    },
    KeySpec {
        id: "t",
        label: "T",
        width: 30.0,
    },
    KeySpec {
        id: "y",
        label: "Y",
        width: 30.0,
    },
    KeySpec {
        id: "u",
        label: "U",
        width: 30.0,
    },
    KeySpec {
        id: "i",
        label: "I",
        width: 30.0,
    },
    KeySpec {
        id: "o",
        label: "O",
        width: 30.0,
    },
    KeySpec {
        id: "p",
        label: "P",
        width: 30.0,
    },
    KeySpec {
        id: "left_bracket",
        label: "[",
        width: 30.0,
    },
    KeySpec {
        id: "right_bracket",
        label: "]",
        width: 30.0,
    },
    KeySpec {
        id: "backslash",
        label: "| \\",
        width: 81.0,
    },
];

const ROW_4: [KeySpec; 12] = [
    KeySpec {
        id: "caps_lock",
        label: "Caps Lock",
        width: 85.0,
    },
    KeySpec {
        id: "a",
        label: "A",
        width: 30.0,
    },
    KeySpec {
        id: "s",
        label: "S",
        width: 30.0,
    },
    KeySpec {
        id: "d",
        label: "D",
        width: 30.0,
    },
    KeySpec {
        id: "f",
        label: "F",
        width: 30.0,
    },
    KeySpec {
        id: "g",
        label: "G",
        width: 30.0,
    },
    KeySpec {
        id: "h",
        label: "H",
        width: 30.0,
    },
    KeySpec {
        id: "j",
        label: "J",
        width: 30.0,
    },
    KeySpec {
        id: "k",
        label: "K",
        width: 30.0,
    },
    KeySpec {
        id: "l",
        label: "L",
        width: 30.0,
    },
    KeySpec {
        id: "semicolon",
        label: ";",
        width: 30.0,
    },
    KeySpec {
        id: "enter",
        label: "Enter",
        width: 130.0,
    },
];

const ROW_5: [KeySpec; 12] = [
    KeySpec {
        id: "shift_left",
        label: "Shift",
        width: 95.0,
    },
    KeySpec {
        id: "z",
        label: "Z",
        width: 30.0,
    },
    KeySpec {
        id: "x",
        label: "X",
        width: 30.0,
    },
    KeySpec {
        id: "c",
        label: "C",
        width: 30.0,
    },
    KeySpec {
        id: "v",
        label: "V",
        width: 30.0,
    },
    KeySpec {
        id: "b",
        label: "B",
        width: 30.0,
    },
    KeySpec {
        id: "n",
        label: "N",
        width: 30.0,
    },
    KeySpec {
        id: "m",
        label: "M",
        width: 30.0,
    },
    KeySpec {
        id: "comma",
        label: ",",
        width: 30.0,
    },
    KeySpec {
        id: "period",
        label: ".",
        width: 30.0,
    },
    KeySpec {
        id: "slash",
        label: "/",
        width: 30.0,
    },
    KeySpec {
        id: "shift_right",
        label: "Shift",
        width: 120.0,
    },
];

const ROW_6: [KeySpec; 8] = [
    KeySpec {
        id: "ctrl_left",
        label: "Ctrl",
        width: 50.0,
    },
    KeySpec {
        id: "win_left",
        label: "Win",
        width: 50.0,
    },
    KeySpec {
        id: "alt_left",
        label: "Alt",
        width: 50.0,
    },
    KeySpec {
        id: "space",
        label: "",
        width: 135.0,
    },
    KeySpec {
        id: "alt_right",
        label: "Alt",
        width: 50.0,
    },
    KeySpec {
        id: "win_right",
        label: "Win",
        width: 50.0,
    },
    KeySpec {
        id: "menu",
        label: "Menu",
        width: 50.0,
    },
    KeySpec {
        id: "ctrl_right",
        label: "Ctrl",
        width: 50.0,
    },
];

const TKL_MAIN_ROW_1: &[KeySpec] = &[
    KeySpec {
        id: "grave",
        label: "`",
        width: 38.0,
    },
    KeySpec {
        id: "1",
        label: "1",
        width: 38.0,
    },
    KeySpec {
        id: "2",
        label: "2",
        width: 38.0,
    },
    KeySpec {
        id: "3",
        label: "3",
        width: 38.0,
    },
    KeySpec {
        id: "4",
        label: "4",
        width: 38.0,
    },
    KeySpec {
        id: "5",
        label: "5",
        width: 38.0,
    },
    KeySpec {
        id: "6",
        label: "6",
        width: 38.0,
    },
    KeySpec {
        id: "7",
        label: "7",
        width: 38.0,
    },
    KeySpec {
        id: "8",
        label: "8",
        width: 38.0,
    },
    KeySpec {
        id: "9",
        label: "9",
        width: 38.0,
    },
    KeySpec {
        id: "0",
        label: "0",
        width: 38.0,
    },
    KeySpec {
        id: "minus",
        label: "-",
        width: 38.0,
    },
    KeySpec {
        id: "equal",
        label: "=",
        width: 38.0,
    },
    KeySpec {
        id: "backspace",
        label: "Backspace",
        width: 72.0,
    },
];

const TKL_MAIN_ROW_2: &[KeySpec] = &[
    KeySpec {
        id: "tab",
        label: "Tab",
        width: 58.0,
    },
    KeySpec {
        id: "q",
        label: "Q",
        width: 38.0,
    },
    KeySpec {
        id: "w",
        label: "W",
        width: 38.0,
    },
    KeySpec {
        id: "e",
        label: "E",
        width: 38.0,
    },
    KeySpec {
        id: "r",
        label: "R",
        width: 38.0,
    },
    KeySpec {
        id: "t",
        label: "T",
        width: 38.0,
    },
    KeySpec {
        id: "y",
        label: "Y",
        width: 38.0,
    },
    KeySpec {
        id: "u",
        label: "U",
        width: 38.0,
    },
    KeySpec {
        id: "i",
        label: "I",
        width: 38.0,
    },
    KeySpec {
        id: "o",
        label: "O",
        width: 38.0,
    },
    KeySpec {
        id: "p",
        label: "P",
        width: 38.0,
    },
    KeySpec {
        id: "left_bracket",
        label: "[",
        width: 38.0,
    },
    KeySpec {
        id: "right_bracket",
        label: "]",
        width: 38.0,
    },
    KeySpec {
        id: "backslash",
        label: "\\",
        width: 58.0,
    },
];

const TKL_MAIN_ROW_3: &[KeySpec] = &[
    KeySpec {
        id: "caps_lock",
        label: "Caps Lock",
        width: 72.0,
    },
    KeySpec {
        id: "a",
        label: "A",
        width: 38.0,
    },
    KeySpec {
        id: "s",
        label: "S",
        width: 38.0,
    },
    KeySpec {
        id: "d",
        label: "D",
        width: 38.0,
    },
    KeySpec {
        id: "f",
        label: "F",
        width: 38.0,
    },
    KeySpec {
        id: "g",
        label: "G",
        width: 38.0,
    },
    KeySpec {
        id: "h",
        label: "H",
        width: 38.0,
    },
    KeySpec {
        id: "j",
        label: "J",
        width: 38.0,
    },
    KeySpec {
        id: "k",
        label: "K",
        width: 38.0,
    },
    KeySpec {
        id: "l",
        label: "L",
        width: 38.0,
    },
    KeySpec {
        id: "semicolon",
        label: ";",
        width: 38.0,
    },
    KeySpec {
        id: "quote",
        label: "'",
        width: 38.0,
    },
    KeySpec {
        id: "enter",
        label: "Enter",
        width: 78.0,
    },
];

const TKL_MAIN_ROW_4: &[KeySpec] = &[
    KeySpec {
        id: "left_shift",
        label: "Shift",
        width: 92.0,
    },
    KeySpec {
        id: "z",
        label: "Z",
        width: 38.0,
    },
    KeySpec {
        id: "x",
        label: "X",
        width: 38.0,
    },
    KeySpec {
        id: "c",
        label: "C",
        width: 38.0,
    },
    KeySpec {
        id: "v",
        label: "V",
        width: 38.0,
    },
    KeySpec {
        id: "b",
        label: "B",
        width: 38.0,
    },
    KeySpec {
        id: "n",
        label: "N",
        width: 38.0,
    },
    KeySpec {
        id: "m",
        label: "M",
        width: 38.0,
    },
    KeySpec {
        id: "comma",
        label: ",",
        width: 38.0,
    },
    KeySpec {
        id: "period",
        label: ".",
        width: 38.0,
    },
    KeySpec {
        id: "slash",
        label: "/",
        width: 38.0,
    },
    KeySpec {
        id: "right_shift",
        label: "Shift",
        width: 96.0,
    },
];

const TKL_MAIN_ROW_5: &[KeySpec] = &[
    KeySpec {
        id: "left_ctrl",
        label: "Ctrl",
        width: 58.0,
    },
    KeySpec {
        id: "left_meta",
        label: "Win",
        width: 58.0,
    },
    KeySpec {
        id: "left_alt",
        label: "Alt",
        width: 58.0,
    },
    KeySpec {
        id: "space",
        label: "",
        width: 190.0,
    },
    KeySpec {
        id: "right_alt",
        label: "Alt",
        width: 58.0,
    },
    KeySpec {
        id: "right_meta",
        label: "Win",
        width: 58.0,
    },
    KeySpec {
        id: "menu",
        label: "Menu",
        width: 58.0,
    },
    KeySpec {
        id: "right_ctrl",
        label: "Ctrl",
        width: 58.0,
    },
];

const TKL_NAV_ROW_1: &[KeySpec] = &[
    KeySpec {
        id: "insert",
        label: "Ins",
        width: 38.0,
    },
    KeySpec {
        id: "home",
        label: "Home",
        width: 38.0,
    },
    KeySpec {
        id: "page_up",
        label: "PgUp",
        width: 38.0,
    },
];

const TKL_NAV_ROW_2: &[KeySpec] = &[
    KeySpec {
        id: "delete",
        label: "Del",
        width: 38.0,
    },
    KeySpec {
        id: "end",
        label: "End",
        width: 38.0,
    },
    KeySpec {
        id: "page_down",
        label: "PgDn",
        width: 38.0,
    },
];

const TKL_NAV_ROW_3: &[KeySpec] = &[
    KeySpec {
        id: "nav_up_spacer",
        label: "",
        width: 38.0,
    },
    KeySpec {
        id: "arrow_up",
        label: "↑",
        width: 38.0,
    },
    KeySpec {
        id: "nav_up_spacer_2",
        label: "",
        width: 38.0,
    },
];

const TKL_NAV_ROW_4: &[KeySpec] = &[
    KeySpec {
        id: "arrow_left",
        label: "←",
        width: 38.0,
    },
    KeySpec {
        id: "arrow_down",
        label: "↓",
        width: 38.0,
    },
    KeySpec {
        id: "arrow_right",
        label: "→",
        width: 38.0,
    },
];

const FULL_NUMPAD_ROW_1: &[KeySpec] = &[
    KeySpec {
        id: "num_lock",
        label: "Num",
        width: 38.0,
    },
    KeySpec {
        id: "num_divide",
        label: "/",
        width: 38.0,
    },
    KeySpec {
        id: "num_multiply",
        label: "*",
        width: 38.0,
    },
    KeySpec {
        id: "num_minus",
        label: "-",
        width: 38.0,
    },
];

const FULL_NUMPAD_ROW_2: &[KeySpec] = &[
    KeySpec {
        id: "num_7",
        label: "7",
        width: 38.0,
    },
    KeySpec {
        id: "num_8",
        label: "8",
        width: 38.0,
    },
    KeySpec {
        id: "num_9",
        label: "9",
        width: 38.0,
    },
    KeySpec {
        id: "num_plus_1",
        label: "+",
        width: 38.0,
    },
];

const FULL_NUMPAD_ROW_3: &[KeySpec] = &[
    KeySpec {
        id: "num_4",
        label: "4",
        width: 38.0,
    },
    KeySpec {
        id: "num_5",
        label: "5",
        width: 38.0,
    },
    KeySpec {
        id: "num_6",
        label: "6",
        width: 38.0,
    },
    KeySpec {
        id: "num_plus_2",
        label: "+",
        width: 38.0,
    },
];

const FULL_NUMPAD_ROW_4: &[KeySpec] = &[
    KeySpec {
        id: "num_1",
        label: "1",
        width: 38.0,
    },
    KeySpec {
        id: "num_2",
        label: "2",
        width: 38.0,
    },
    KeySpec {
        id: "num_3",
        label: "3",
        width: 38.0,
    },
    KeySpec {
        id: "num_enter_1",
        label: "Enter",
        width: 38.0,
    },
];

const FULL_NUMPAD_ROW_5: &[KeySpec] = &[
    KeySpec {
        id: "num_0",
        label: "0",
        width: 78.0,
    },
    KeySpec {
        id: "num_decimal",
        label: ".",
        width: 38.0,
    },
    KeySpec {
        id: "num_enter_2",
        label: "Enter",
        width: 38.0,
    },
];

const TKL_SYSTEM_ROW: &[KeySpec] = &[KeySpec {
    id: "print_screen",
    label: "PrtSc",
    width: 122.0,
}];

const TKL_FUNCTION_ROW: [KeySpec; 13] = [
    KeySpec {
        id: "esc",
        label: "Esc",
        width: 44.0,
    },
    KeySpec {
        id: "f1",
        label: "F1",
        width: 44.0,
    },
    KeySpec {
        id: "f2",
        label: "F2",
        width: 44.0,
    },
    KeySpec {
        id: "f3",
        label: "F3",
        width: 44.0,
    },
    KeySpec {
        id: "f4",
        label: "F4",
        width: 44.0,
    },
    KeySpec {
        id: "f5",
        label: "F5",
        width: 44.0,
    },
    KeySpec {
        id: "f6",
        label: "F6",
        width: 44.0,
    },
    KeySpec {
        id: "f7",
        label: "F7",
        width: 44.0,
    },
    KeySpec {
        id: "f8",
        label: "F8",
        width: 44.0,
    },
    KeySpec {
        id: "f9",
        label: "F9",
        width: 44.0,
    },
    KeySpec {
        id: "f10",
        label: "F10",
        width: 44.0,
    },
    KeySpec {
        id: "f11",
        label: "F11",
        width: 44.0,
    },
    KeySpec {
        id: "f12",
        label: "F12",
        width: 44.0,
    },
];

pub struct KeyboardEditorView {
    preset_id: Option<SharedString>,
    preset_name: SharedString,
    name_input: Entity<InputState>,
    _name_subscription: Subscription,
    initial_data: KeyboardPresetData,

    selected_keys: [Vec<&'static str>; 3],
    sync_selections: bool,

    files: Vec<AudioFile>,
    next_file_id: u64,
    upload_tasks: HashMap<u64, Task<()>>,

    hovered_file: Option<u64>,

    playback_mode: PlaybackMode,
    preferred_playback_mode: PlaybackMode,
    keyboard_layout: KeyboardLayout,
    save_status: SaveStatus,
}

impl EventEmitter<KeyboardEditorEvent> for KeyboardEditorView {}

impl KeyboardEditorView {
    pub fn view(
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<KeyboardPresetData>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(preset_id, preset_name, preset_data, window, cx))
    }

    pub fn set_preview_state(&mut self, file_id: u64, state: PreviewState, cx: &mut Context<Self>) {
        if let Some(file) = self.files.iter_mut().find(|file| file.id == file_id) {
            file.preview_state = state.clone();
            cx.emit(KeyboardEditorEvent::PlaybackStateChanged { state });
            cx.notify();
        }
    }

    pub fn set_save_status(&mut self, status: SaveStatus, cx: &mut Context<Self>) {
        self.save_status = status;
        cx.notify();
    }

    fn new(
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<KeyboardPresetData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Preset name")
                .default_value(preset_name.as_ref())
        });
        let name_subscription =
            cx.subscribe_in(&name_input, window, |_, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            });
        let mut initial_data = preset_data.unwrap_or(KeyboardPresetData {
            selected_keys: [vec![], vec![], vec![]],
            files: Vec::new(),
            random_playback: false,
            layout_index: KeyboardLayout::Compact.index(),
            sync_selections: false,
        });
        let selected_audio_file_count = initial_data
            .files
            .iter()
            .filter(|file| file.selected)
            .count();
        if selected_audio_file_count < 2 {
            initial_data.random_playback = false;
        }
        let selected_keys = initial_data.selected_keys.clone();
        let files = initial_data
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| AudioFile {
                id: index as u64,
                path: file.path.clone(),
                name: file.name.clone(),
                size: file.size.clone(),
                selected: file.selected,
                state: UploadState::Complete,
                preview_state: PreviewState::Stopped,
            })
            .collect::<Vec<_>>();
        let playback_mode = if initial_data.random_playback {
            PlaybackMode::Random
        } else {
            PlaybackMode::Sequential
        };
        if selected_audio_file_count < 2 {
            println!("Keyboard playback mode: None");
        } else {
            println!(
                "Keyboard playback mode: {}",
                match playback_mode {
                    PlaybackMode::Sequential => "Sequential",
                    PlaybackMode::Random => "Random",
                }
            );
        }
        let next_file_id = files.len() as u64;
        let keyboard_layout = match initial_data.layout_index {
            0 => KeyboardLayout::FullSize,
            1 => KeyboardLayout::Tkl,
            _ => KeyboardLayout::Compact,
        };
        let sync_selections = initial_data.sync_selections;

        Self {
            preset_id,
            preset_name,
            name_input,
            _name_subscription: name_subscription,
            initial_data,
            selected_keys,
            sync_selections,
            files,
            next_file_id,
            upload_tasks: HashMap::new(),
            hovered_file: None,
            playback_mode,
            preferred_playback_mode: playback_mode,
            keyboard_layout,
            save_status: SaveStatus::Idle,
        }
    }

    fn render_keyboard(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.keyboard_layout {
            KeyboardLayout::FullSize => self.render_full_size_keyboard(cx).into_any_element(),
            KeyboardLayout::Tkl => self.render_tkl_keyboard(cx).into_any_element(),
            KeyboardLayout::Compact => self.render_compact_keyboard(cx).into_any_element(),
        }
    }

    fn render_keyboard_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let width = self.keyboard_layout.width();

        div()
            .id("keyboard-horizontal-scroll")
            .w_full()
            .min_w_0()
            .overflow_x_scrollbar()
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(width))
                    .pb(px(12.))
                    .child(self.render_keyboard(cx)),
            )
    }

    fn is_selected(&self, key_id: &str) -> bool {
        self.selected_keys[self.keyboard_layout.index()].contains(&key_id)
    }

    fn are_keys_selected(&self, keys: &[&'static str]) -> bool {
        let selected_keys = &self.selected_keys[self.keyboard_layout.index()];
        !keys.is_empty() && keys.iter().all(|key| selected_keys.contains(key))
    }

    fn selection_groups(&self) -> Vec<SelectionGroup> {
        const LETTER_KEYS: [&str; 26] = [
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z",
        ];
        const NUMBER_ROW_KEYS: [&str; 13] = [
            "grave", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "minus", "equal",
        ];
        const NAVIGATION_KEYS: [&str; 10] = [
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

        let mut groups = vec![SelectionGroup {
            id: "all-keys",
            label: "All keys",
            keys: self.keyboard_layout.key_ids(),
        }];
        groups.push(SelectionGroup {
            id: "letters",
            label: "A–Z",
            keys: LETTER_KEYS.to_vec(),
        });
        groups.push(SelectionGroup {
            id: "number-row",
            label: "Number row",
            keys: NUMBER_ROW_KEYS.to_vec(),
        });
        groups.push(SelectionGroup {
            id: "function-row",
            label: "Function row",
            keys: TKL_FUNCTION_ROW.iter().map(|key| key.id).collect(),
        });

        let modifier_keys = match self.keyboard_layout {
            KeyboardLayout::Compact => vec![
                "shift_left",
                "shift_right",
                "ctrl_left",
                "ctrl_right",
                "win_left",
                "win_right",
                "alt_left",
                "alt_right",
            ],
            KeyboardLayout::Tkl | KeyboardLayout::FullSize => vec![
                "left_shift",
                "right_shift",
                "left_ctrl",
                "right_ctrl",
                "left_meta",
                "right_meta",
                "left_alt",
                "right_alt",
            ],
        };
        groups.push(SelectionGroup {
            id: "modifiers",
            label: "Modifiers",
            keys: modifier_keys,
        });

        if self.keyboard_layout != KeyboardLayout::Compact {
            groups.push(SelectionGroup {
                id: "navigation",
                label: "Navigation keys",
                keys: NAVIGATION_KEYS.to_vec(),
            });
        }

        if self.keyboard_layout == KeyboardLayout::FullSize {
            let mut numpad_keys = Vec::new();
            for row in [
                FULL_NUMPAD_ROW_1,
                FULL_NUMPAD_ROW_2,
                FULL_NUMPAD_ROW_3,
                FULL_NUMPAD_ROW_4,
                FULL_NUMPAD_ROW_5,
            ] {
                numpad_keys.extend(row.iter().map(|key| key.id));
            }
            groups.push(SelectionGroup {
                id: "numpad",
                label: "Numpad",
                keys: numpad_keys,
            });
        }

        groups
    }

    fn toggle_selection_group(
        &mut self,
        group_id: &'static str,
        keys: &[&'static str],
        cx: &mut Context<Self>,
    ) {
        let group_is_selected = self.are_keys_selected(keys);

        if self.sync_selections {
            let canonical_keys = keys
                .iter()
                .map(|key| canonical_key_id(key))
                .collect::<Vec<_>>();

            for layout in KeyboardLayout::ALL {
                let matching_keys = if group_id == "all-keys" {
                    layout.key_ids()
                } else {
                    layout
                        .key_ids()
                        .into_iter()
                        .filter(|key| canonical_keys.contains(&canonical_key_id(key)))
                        .collect::<Vec<_>>()
                };
                let selected_keys = &mut self.selected_keys[layout.index()];

                if group_is_selected {
                    selected_keys.retain(|key| !matching_keys.contains(key));
                } else {
                    for key in matching_keys {
                        if !selected_keys.contains(&key) {
                            selected_keys.push(key);
                        }
                    }
                }
            }
        } else {
            let selected_keys = &mut self.selected_keys[self.keyboard_layout.index()];
            if group_is_selected {
                selected_keys.retain(|key| !keys.contains(key));
            } else {
                for key in keys {
                    if !selected_keys.contains(key) {
                        selected_keys.push(*key);
                    }
                }
            }
        }

        cx.notify();
    }

    fn clear_selected_keys(&mut self, cx: &mut Context<Self>) {
        if self.sync_selections {
            for selected_keys in &mut self.selected_keys {
                selected_keys.clear();
            }
        } else {
            self.selected_keys[self.keyboard_layout.index()].clear();
        }
        cx.notify();
    }

    fn has_unsaved_changes(&self, cx: &App) -> bool {
        let current_data = self.current_preset_data();
        let selected_keys_unchanged = current_data
            .selected_keys
            .iter()
            .zip(&self.initial_data.selected_keys)
            .all(|(current, initial)| {
                current.iter().copied().collect::<BTreeSet<_>>()
                    == initial.iter().copied().collect::<BTreeSet<_>>()
            });
        let data_unchanged = selected_keys_unchanged
            && current_data.files == self.initial_data.files
            && current_data.random_playback == self.initial_data.random_playback
            && current_data.layout_index == self.initial_data.layout_index
            && current_data.sync_selections == self.initial_data.sync_selections;

        !data_unchanged || self.name_input.read(cx).value() != self.preset_name.as_ref()
    }

    fn mapped_key_count(&self) -> usize {
        self.selected_keys
            .iter()
            .flatten()
            .map(|key| canonical_key_id(key).to_string())
            .collect::<BTreeSet<_>>()
            .len()
    }

    fn current_preset_data(&self) -> KeyboardPresetData {
        KeyboardPresetData {
            selected_keys: self.selected_keys.clone(),
            files: self
                .files
                .iter()
                .map(|file| KeyboardSoundData {
                    path: file.path.clone(),
                    name: file.name.clone(),
                    size: file.size.clone(),
                    selected: file.selected,
                })
                .collect(),
            random_playback: self.playback_mode == PlaybackMode::Random,
            layout_index: self.keyboard_layout.index(),
            sync_selections: self.sync_selections,
        }
    }

    fn save_validation_error(&self, name: &str) -> Option<SharedString> {
        if let Err(message) = crate::storage::validate_preset_name(name) {
            return Some(message.into());
        }

        if self.mapped_key_count() == 0 {
            return Some("Select at least one key to map.".into());
        }

        let has_selected_sound = self.files.iter().any(|file| {
            file.selected && matches!(file.state, UploadState::Success | UploadState::Complete)
        });
        if !has_selected_sound {
            return Some("Select at least one successfully uploaded sound.".into());
        }

        None
    }

    fn set_sync_selections(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if enabled && !self.sync_selections {
            let current_keys = self.selected_keys[self.keyboard_layout.index()].clone();
            let canonical_keys = current_keys
                .iter()
                .map(|key| canonical_key_id(key))
                .collect::<Vec<_>>();

            for layout in KeyboardLayout::ALL {
                let layout_keys = layout.key_ids();
                self.selected_keys[layout.index()] = layout_keys
                    .into_iter()
                    .filter(|key| canonical_keys.contains(&canonical_key_id(key)))
                    .collect();
            }
        } else if !enabled && self.sync_selections {
            let active_layout = self.keyboard_layout.index();
            for layout in KeyboardLayout::ALL {
                if layout.index() != active_layout {
                    self.selected_keys[layout.index()].clear();
                }
            }
        }

        self.sync_selections = enabled;
        cx.notify();
    }

    fn has_file_name(&self, name: &str) -> bool {
        self.files
            .iter()
            .any(|file| file.name.as_ref().eq_ignore_ascii_case(name))
    }

    fn toggle_key(&mut self, key_id: &'static str, cx: &mut Context<Self>) {
        toggle_key_selection(
            &mut self.selected_keys,
            self.keyboard_layout,
            key_id,
            self.sync_selections,
        );

        cx.notify();
    }

    fn apply_layout_change(&mut self, next_layout: KeyboardLayout) -> bool {
        if next_layout == self.keyboard_layout {
            return false;
        }

        if !self.sync_selections {
            self.selected_keys[self.keyboard_layout.index()].clear();
        }

        self.keyboard_layout = next_layout;
        true
    }

    fn set_layout(&mut self, index: usize, cx: &mut Context<Self>) {
        let next_layout = match index {
            0 => KeyboardLayout::FullSize,
            1 => KeyboardLayout::Tkl,
            _ => KeyboardLayout::Compact,
        };

        self.apply_layout_change(next_layout);

        println!(
            "Keyboard layout: {}",
            match self.keyboard_layout {
                KeyboardLayout::FullSize => "Full size",
                KeyboardLayout::Tkl => "TKL",
                KeyboardLayout::Compact => "Compact",
            }
        );

        cx.notify();
    }

    fn set_playback_mode(&mut self, mode: PlaybackMode, cx: &mut Context<Self>) {
        if mode == PlaybackMode::Random && !self.random_playback_available() {
            return;
        }

        self.playback_mode = mode;
        self.preferred_playback_mode = mode;

        println!(
            "Keyboard playback mode: {}",
            match mode {
                PlaybackMode::Sequential => "Sequential",
                PlaybackMode::Random => "Random",
            }
        );

        cx.notify();
    }

    fn random_playback_available(&self) -> bool {
        self.selected_audio_file_count() >= 2
    }

    fn selected_audio_file_count(&self) -> usize {
        self.files.iter().filter(|file| file.selected).count()
    }

    fn update_playback_mode_for_selection_change(&mut self, previous_count: usize) {
        let current_count = self.selected_audio_file_count();
        if current_count == previous_count {
            return;
        }

        if current_count < 2 {
            self.playback_mode = PlaybackMode::Sequential;
            println!("Keyboard playback mode: None");
        } else if previous_count < 2 {
            self.playback_mode = self.preferred_playback_mode;
            println!(
                "Keyboard playback mode: {}",
                match self.playback_mode {
                    PlaybackMode::Sequential => "Sequential",
                    PlaybackMode::Random => "Random",
                }
            );
        }
    }

    fn format_file_size(bytes: u64) -> SharedString {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        if bytes >= GB as u64 {
            format!("{:.1} GB", bytes as f64 / GB).into()
        } else if bytes >= MB as u64 {
            format!("{:.1} MB", bytes as f64 / MB).into()
        } else if bytes >= KB as u64 {
            format!("{:.0} KB", bytes as f64 / KB).into()
        } else {
            format!("{bytes} B").into()
        }
    }

    fn is_audio_file(path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                AUDIO_EXTENSIONS
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(extension))
            })
            .unwrap_or(false)
    }

    fn begin_uploads(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        for path in paths {
            if !Self::is_audio_file(&path) {
                continue;
            }

            let Some(name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(SharedString::from)
            else {
                continue;
            };

            if self.has_file_name(name.as_ref()) {
                println!("Duplicate file upload rejected: {}", name);

                window.push_notification(
                    Notification::error(format!("\"{}\" is already in the file list.", name))
                        .title("Duplicate file")
                        .placement(Anchor::BottomRight)
                        .autohide(true)
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.notify();
                            cx.hide();
                        })),
                    cx,
                );
                continue;
            }

            let id = self.next_file_id;
            self.next_file_id += 1;

            self.files.push(AudioFile::new(id, path.clone(), name));

            let task = cx.spawn(async move |this, cx| {
                let metadata = cx
                    .background_spawn(async move {
                        std::fs::metadata(&path).map(|metadata| metadata.len())
                    })
                    .await;

                let Ok(size) = metadata else {
                    _ = this.update(cx, |view, cx| {
                        if let Some(file) = view.files.iter_mut().find(|file| file.id == id) {
                            file.state = UploadState::Failed;
                        }

                        cx.notify();
                    });

                    return;
                };

                if this
                    .update(cx, |view, cx| {
                        if let Some(file) = view.files.iter_mut().find(|file| file.id == id) {
                            file.size = Some(Self::format_file_size(size));
                            file.state = UploadState::Success;
                        }

                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }

                cx.background_executor()
                    .timer(Duration::from_millis(450))
                    .await;

                _ = this.update(cx, |view, cx| {
                    if let Some(file) = view.files.iter_mut().find(|file| file.id == id) {
                        file.state = UploadState::Complete;
                    }

                    cx.notify();
                });
            });

            self.upload_tasks.insert(id, task);
        }

        cx.notify();
    }

    fn open_file_picker(&self, window: &mut Window, cx: &mut Context<Self>) {
        let window_handle = window.window_handle();

        cx.spawn(async move |this, cx| {
            let files = rfd::AsyncFileDialog::new()
                .set_title("Select audio files")
                .add_filter("Audio files", AUDIO_EXTENSIONS)
                .pick_files()
                .await;

            let Some(files) = files else {
                return;
            };

            let paths = files
                .into_iter()
                .map(|file| file.path().to_path_buf())
                .collect::<Vec<_>>();

            _ = window_handle.update(cx, |_, window, cx| {
                _ = this.update(cx, |view, cx| {
                    if !paths.is_empty() {
                        cx.emit(KeyboardEditorEvent::StopPreviewRequested { file_id: None });
                    }
                    view.begin_uploads(paths, window, cx);
                });
            });
        })
        .detach();
    }

    fn remove_file(&mut self, id: u64, cx: &mut Context<Self>) {
        let Some(index) = self.files.iter().position(|file| file.id == id) else {
            return;
        };

        let previous_count = self.selected_audio_file_count();
        if self.files[index].preview_state == PreviewState::Playing {
            cx.emit(KeyboardEditorEvent::StopPreviewRequested { file_id: Some(id) });
        }
        let file = self.files.remove(index);

        let was_uploading = file.state == UploadState::Uploading;

        self.upload_tasks.remove(&id);

        if self.hovered_file == Some(id) {
            self.hovered_file = None;
        }

        self.update_playback_mode_for_selection_change(previous_count);

        if was_uploading {
            println!("Upload cancelled and file removed: {}", file.name);
        } else {
            println!("File removed: {}", file.name);
        }

        cx.notify();
    }

    fn toggle_file_selection(&mut self, id: u64, state: CheckboxState, cx: &mut Context<Self>) {
        let previous_count = self.selected_audio_file_count();
        if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
            file.selected = state == CheckboxState::Checked;

            println!(
                "{}: {}",
                file.name,
                if file.selected {
                    "selected"
                } else {
                    "unselected"
                }
            );
        }

        self.update_playback_mode_for_selection_change(previous_count);

        cx.notify();
    }

    fn render_key_row(&self, row: &[KeySpec], cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().w_full().gap_1().children(row.iter().map(|key| {
            if key.id.starts_with("nav_up_spacer") {
                return div().w(px(key.width)).h(px(34.)).into_any_element();
            }

            let selected = self.is_selected(key.id);
            let key_id = key.id;

            let button_variant = if selected {
                ButtonCustomVariant::new(cx)
                    .color(cx.theme().blue.opacity(0.08))
                    .foreground(cx.theme().blue)
                    .hover(cx.theme().blue.opacity(0.08))
                    .active(cx.theme().blue.opacity(0.12))
            } else {
                ButtonCustomVariant::new(cx)
                    .color(cx.theme().background)
                    .foreground(cx.theme().muted_foreground)
                    .hover(cx.theme().background)
                    .active(cx.theme().background)
            };

            Button::new(key.id)
                .custom(button_variant)
                .border_1()
                .border_color(if selected {
                    cx.theme().blue
                } else {
                    cx.theme().muted_foreground.opacity(0.3)
                })
                .w(px(key.width))
                .h(px(34.))
                .px_0()
                .text_size(px(12.))
                .label(key.label)
                .on_click(cx.listener(move |this, _event, _window, cx| {
                    this.toggle_key(key_id, cx);
                }))
                .into_any_element()
        }))
    }

    fn render_compact_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_1()
            .child(self.render_key_row(&ROW_1, cx))
            .child(self.render_key_row(&ROW_2, cx))
            .child(self.render_key_row(&ROW_3, cx))
            .child(self.render_key_row(&ROW_4, cx))
            .child(self.render_key_row(&ROW_5, cx))
            .child(self.render_key_row(&ROW_6, cx))
    }

    fn render_tkl_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .w(px(770.0))
            .items_start()
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(&TKL_FUNCTION_ROW, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_1, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_2, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_3, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_4, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_5, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(TKL_SYSTEM_ROW, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_1, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_2, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_3, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_4, cx)),
            )
    }

    fn render_full_size_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .w(px(950.0))
            .items_start()
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(&TKL_FUNCTION_ROW, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_1, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_2, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_3, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_4, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_5, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(TKL_SYSTEM_ROW, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_1, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_2, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_3, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_4, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(FULL_NUMPAD_ROW_1, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_2, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_3, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_4, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_5, cx)),
            )
    }

    fn render_hint(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
        div()
            .px_2()
            .py(px(1.))
            .rounded(px(4.))
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.3))
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(text.into())
    }

    fn render_kbd_hint(keystroke: &str, cx: &App) -> impl IntoElement {
        div()
            .px_2()
            .py(px(1.))
            .rounded(px(4.))
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.3))
            .bg(cx.theme().background)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(
                Kbd::new(Keystroke::parse(keystroke).unwrap())
                    .appearance(false)
                    .text_size(px(12.)),
            )
    }

    fn render_selection_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let groups = self.selection_groups();

        h_flex()
            .w_full()
            .flex_wrap()
            .gap_1()
            .children(groups.into_iter().map(|group| {
                let active = self.are_keys_selected(&group.keys);
                let keys = group.keys;
                let group_id = group.id;
                let id = format!("quick-select-{}", group.id);

                Button::new(id)
                    .label(group.label)
                    .compact()
                    .selected(active)
                    .when(active, |button| button.primary())
                    .when(!active, |button| button.outline())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_selection_group(group_id, &keys, cx);
                    }))
                    .into_any_element()
            }))
            .child(
                Button::new("clear-selected-keys")
                    .danger()
                    .outline()
                    .compact()
                    .label("Clear selected keys")
                    .disabled(self.selected_keys[self.keyboard_layout.index()].is_empty())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clear_selected_keys(cx);
                    })),
            )
    }

    fn render_file_row(&self, file: AudioFile, cx: &mut Context<Self>) -> impl IntoElement {
        let id = file.id;
        let checked = file.selected;
        let hovered = self.hovered_file == Some(id);
        let entity = cx.entity().downgrade();

        let media_content = match file.state {
            UploadState::Uploading => Spinner::new()
                .small()
                .color(cx.theme().blue)
                .into_any_element(),

            UploadState::Success => Icon::new(IconName::CircleCheck)
                .with_size(px(18.))
                .text_color(cx.theme().green)
                .into_any_element(),

            UploadState::Complete => {
                let preview_entity = entity.clone();
                let path = file.path.clone();
                let is_playing = file.preview_state == PreviewState::Playing;
                let preview_button = Button::new(format!("preview-file-{id}"))
                    .ghost()
                    .xsmall()
                    .icon(if is_playing {
                        IconName::Close
                    } else if hovered {
                        IconName::Play
                    } else {
                        IconName::FileVolume
                    })
                    .accessibility_label(if is_playing {
                        format!("Stop preview of {}", file.name)
                    } else {
                        format!("Play preview of {}", file.name)
                    })
                    .tooltip(if is_playing {
                        "Stop preview"
                    } else {
                        "Play preview"
                    })
                    .on_click(move |_event, _window, cx| {
                        _ = preview_entity.update(cx, |_, cx| {
                            if is_playing {
                                cx.emit(KeyboardEditorEvent::StopPreviewRequested {
                                    file_id: Some(id),
                                });
                            } else {
                                cx.emit(KeyboardEditorEvent::PreviewRequested {
                                    file_id: id,
                                    path: path.clone(),
                                });
                            }
                        });
                    });

                div()
                    .id(format!("preview-hover-{id}"))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_hover({
                        let entity = entity.clone();

                        move |is_hovered, _window, cx| {
                            _ = entity.update(cx, |view, cx| {
                                if *is_hovered {
                                    view.hovered_file = Some(id);
                                } else if view.hovered_file == Some(id) {
                                    view.hovered_file = None;
                                }
                                cx.notify();
                            });
                        }
                    })
                    .child(preview_button)
                    .into_any_element()
            }

            UploadState::Failed => Icon::new(IconName::CircleX)
                .with_size(px(18.))
                .text_color(cx.theme().red)
                .into_any_element(),
        };

        let media = AttachmentMedia::new().child(media_content);

        let attachment_status = match file.state {
            UploadState::Uploading => AttachmentStatus::Uploading,
            UploadState::Success | UploadState::Complete => AttachmentStatus::Complete,
            UploadState::Failed => AttachmentStatus::Failed,
        };

        let description = match file.state {
            UploadState::Uploading => SharedString::from("Uploading"),
            UploadState::Success | UploadState::Complete => match &file.preview_state {
                PreviewState::Error(message) => message.clone(),
                _ => file.size.clone().unwrap_or_else(|| "Preparing...".into()),
            },
            UploadState::Failed => SharedString::from("Upload failed"),
        };

        let remove_entity = cx.entity().downgrade();
        let checkbox_entity = cx.entity().downgrade();

        let attachment = Attachment::new()
            .small()
            .status(attachment_status)
            .flex_1()
            .media(AttachmentMedia::new().child(media))
            .content(
                AttachmentContent::new()
                    .title(AttachmentTitle::new(file.name.clone()))
                    .description(AttachmentDescription::new(description)),
            )
            .actions(
                AttachmentActions::new().child(
                    Button::new(format!("remove-file-{id}"))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Close)
                        .accessibility_label(format!("Remove {}", file.name))
                        .tooltip("Remove")
                        .on_click(move |_event, _window, cx| {
                            _ = remove_entity.update(cx, |view, cx| {
                                view.remove_file(id, cx);
                            });
                        }),
                ),
            );

        let checkbox = Checkbox::new(format!("select-file-{id}"))
            .checked(checked)
            .accessibility_label(format!("Select {}", file.name))
            .on_change(move |state, _window, _cx, cx| {
                _ = checkbox_entity.update(cx, |view, cx| {
                    view.toggle_file_selection(id, state, cx);
                });
            })
            .child(
                CheckboxIndicator::new()
                    .checked(checked)
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_4()
                    .border_1()
                    .border_color(cx.theme().muted_foreground.opacity(0.4))
                    .when(checked, |this| {
                        this.bg(cx.theme().blue)
                            .border_color(cx.theme().blue)
                            .child(
                                Icon::new(IconName::Check)
                                    .with_size(px(12.))
                                    .text_color(cx.theme().primary_foreground),
                            )
                    }),
            );

        h_flex()
            .items_center()
            .gap_3()
            .w_full()
            .child(attachment)
            .child(
                div()
                    .w(px(28.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(checkbox),
            )
    }

    fn render_upload_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Empty::new()
            .w_full()
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.25))
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Icon)
                            .child(Icon::new(IconName::Upload).with_size(px(22.))),
                    )
                    .title(EmptyTitle::new().child("Add audio files"))
                    .description(EmptyDescription::new().child("Select one or more audio files.")),
            )
            .content(
                EmptyContent::new().flex_row().justify_center().child(
                    Button::new("upload-files")
                        .primary()
                        .label("Upload Files...")
                        .on_click(cx.listener(|this, _event, window, cx| {
                            this.open_file_picker(window, cx);
                        })),
                ),
            )
    }

    fn render_playback(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let random_playback_available = self.random_playback_available();
        let sequential = Button::new("playback-sequential")
            .disabled(!random_playback_available)
            .label("Sequential")
            .when(
                random_playback_available && self.playback_mode == PlaybackMode::Sequential,
                |this| this.primary(),
            )
            .when(
                random_playback_available && self.playback_mode != PlaybackMode::Sequential,
                |this| this.outline(),
            )
            .when(!random_playback_available, |this| {
                this.ghost().text_color(cx.theme().muted_foreground)
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.set_playback_mode(PlaybackMode::Sequential, cx);
            }));

        let random = Button::new("playback-random")
            .disabled(!random_playback_available)
            .label("Random")
            .when(
                random_playback_available && self.playback_mode == PlaybackMode::Random,
                |this| this.primary(),
            )
            .when(
                random_playback_available && self.playback_mode != PlaybackMode::Random,
                |this| this.outline(),
            )
            .when(!random_playback_available, |this| {
                this.ghost().text_color(cx.theme().muted_foreground)
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.set_playback_mode(PlaybackMode::Random, cx);
            }));

        h_flex().gap_1().child(sequential).child(random)
    }
}

impl Render for KeyboardEditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_count = self.selected_keys[self.keyboard_layout.index()].len();
        let preset_id = self.preset_id.clone();
        let creating = preset_id.is_none();
        let name_input = self.name_input.clone();
        let has_unsaved_changes = self.has_unsaved_changes(cx);

        let files = self.files.clone();
        let mut file_rows = Vec::with_capacity(files.len());

        for file in files {
            file_rows.push(self.render_file_row(file, cx).into_any_element());
        }

        let back_entity = cx.entity().downgrade();
        let back_control = if has_unsaved_changes {
            let confirm_back = back_entity.clone();
            AlertDialog::new(cx)
                .trigger(
                    Button::new("keyboard-editor-back")
                        .outline()
                        .icon(IconName::ArrowLeft)
                        .accessibility_label("Back to home")
                        .tooltip("Back"),
                )
                .on_ok(move |_, _, cx| {
                    _ = confirm_back.update(cx, |_view, cx| {
                        cx.emit(KeyboardEditorEvent::BackRequested);
                    });
                    true
                })
                .content(|content, _, cx| {
                    content
                        .child(
                            DialogHeader::new()
                                .items_center()
                                .child(
                                    Icon::new(IconName::TriangleAlert)
                                        .with_size(px(24.))
                                        .text_color(cx.theme().warning),
                                )
                                .child(
                                    v_flex()
                                        .w_full()
                                        .items_center()
                                        .text_center()
                                        .gap_1()
                                        .child(DialogTitle::new().child("Discard unsaved changes?"))
                                        .child(
                                            DialogDescription::new().child(
                                                "Your keyboard preset changes will be lost.",
                                            ),
                                        ),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    DialogClose::new().child(
                                        Button::new("keyboard-keep-editing")
                                            .outline()
                                            .icon(IconName::Pencil)
                                            .label("Keep editing"),
                                    ),
                                )
                                .child(
                                    DialogAction::new().child(
                                        Button::new("keyboard-discard-changes")
                                            .danger()
                                            .icon(IconName::ArrowLeft)
                                            .label("Discard changes"),
                                    ),
                                ),
                        )
                })
                .into_any_element()
        } else {
            Button::new("keyboard-editor-back")
                .outline()
                .icon(IconName::ArrowLeft)
                .accessibility_label("Back to home")
                .tooltip("Back")
                .on_click(move |_event, _window, cx| {
                    _ = back_entity.update(cx, |_view, cx| {
                        cx.emit(KeyboardEditorEvent::BackRequested);
                    });
                })
                .into_any_element()
        };

        let editor_identity = if creating {
            v_flex()
                .gap_1()
                .child(Label::new("Creating preset").text_color(cx.theme().muted_foreground))
                .into_any_element()
        } else {
            h_flex()
                .gap_1()
                .child(Label::new("Editing").text_color(cx.theme().muted_foreground))
                .child(Label::new(self.preset_name.clone()))
                .into_any_element()
        };

        v_flex().size_full().font_family("Segoe UI").child(
            div().flex_1().min_h_0().overflow_y_scrollbar().child(
                v_flex()
                    .px_6()
                    .py_4()
                    .gap_4()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .child(
                                h_flex().items_center().gap_3().child(back_control).child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            Label::new("Keyboard editor")
                                                .font_semibold()
                                                .text_size(px(17.)),
                                        )
                                        .child(editor_identity),
                                ),
                            )
                            .child(
                                Button::new("save-preset")
                                    .primary()
                                    .label(match &self.save_status {
                                        SaveStatus::Idle => "Save preset",
                                        SaveStatus::Saving => "Saving…",
                                        SaveStatus::Failed(_) => "Retry save",
                                    })
                                    .disabled(
                                        matches!(self.save_status, SaveStatus::Saving)
                                            || !self.has_unsaved_changes(cx),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if matches!(this.save_status, SaveStatus::Saving)
                                            || !this.has_unsaved_changes(cx)
                                        {
                                            return;
                                        }
                                        let preset_name =
                                            name_input.read(cx).value().trim().to_string();
                                        if let Some(message) =
                                            this.save_validation_error(&preset_name)
                                        {
                                            window.push_notification(
                                                Notification::warning(message)
                                                    .title("Cannot save preset")
                                                    .placement(Anchor::BottomRight)
                                                    .autohide(true)
                                                    .on_click(cx.listener(|_, _, _, cx| {
                                                        cx.notify();
                                                        cx.hide();
                                                    })),
                                                cx,
                                            );
                                            return;
                                        }
                                        let mapped_key_count = this.mapped_key_count();
                                        let mapped_key_label = if mapped_key_count == 1 {
                                            "key"
                                        } else {
                                            "keys"
                                        };
                                        cx.emit(KeyboardEditorEvent::SaveRequested {
                                            preset_id: preset_id.clone(),
                                            preset_name: preset_name.into(),
                                            summary: format!(
                                                "Custom · {mapped_key_count} {mapped_key_label} mapped"
                                            )
                                            .into(),
                                            data: this.current_preset_data(),
                                        });
                                    })),
                            ),
                    )
                    .when(creating, |this| {
                        this.child(
                            v_flex()
                                .w_full()
                                .gap_1()
                                .child(Label::new("Preset name").text_sm())
                                .child(
                                    Input::new(&self.name_input)
                                        .w(px(280.))
                                        .id("keyboard-preset-name")
                                        .cleanable(true)
                                        .aria_label("Preset name"),
                                ),
                        )
                    })
                    .child(Separator::horizontal())
                    .child(
                        TabBar::new("keyboard-layout-tabs")
                            .outline()
                            .small()
                            .selected_index(self.keyboard_layout.index())
                            .on_click(cx.listener(|this, index, _, cx| {
                                this.set_layout(*index, cx);
                            }))
                            .child(Tab::new().label("Full size"))
                            .child(Tab::new().label("TKL"))
                            .child(Tab::new().label("Compact")),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(Self::render_hint("Click", cx))
                            .child(
                                Label::new("select key")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(Self::render_kbd_hint("esc", cx))
                            .child(
                                Label::new("clear")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .flex_shrink_0()
                                    .child(Self::render_kbd_hint("shift", cx))
                                    .child(Label::new("+").text_sm())
                                    .child(Self::render_hint("Scroll", cx))
                                    .child(
                                        Label::new("horizontal scroll")
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground),
                                    ),
                            ),
                    )
                    .child(self.render_keyboard_area(cx))
                    .child(Separator::horizontal())
                    .child(self.render_selection_controls(cx))
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_1()
                                    .child(
                                        Label::new("Sync selections across keyboard sizes")
                                            .font_medium(),
                                    )
                                    .child(
                                        Label::new(
                                            "Selecting a key updates its match in every layout.",
                                        )
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                    ),
                            )
                            .child(
                                Switch::new("sync-keyboard-selections")
                                    .checked(self.sync_selections)
                                    .on_change(cx.listener(|this, checked, _, cx| {
                                        this.set_sync_selections(*checked, cx);
                                    })),
                            ),
                    )
                    .child(Separator::horizontal())
                    .child(
                        h_flex().items_center().justify_between().child(
                            v_flex()
                                .gap_1()
                                .child(
                                    Label::new(format!("Editing {} keys", selected_count))
                                        .font_semibold()
                                        .text_size(px(18.)),
                                )
                                .child(
                                    Label::new(format!(
                                        "Files apply to all {} selected keys.",
                                        selected_count
                                    ))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                                ),
                        ),
                    )
                    .child(
                        h_flex().items_start().gap_2().w_full().child(
                            v_flex()
                                .flex_1()
                                .gap_2()
                                .children(file_rows)
                                .child(self.render_upload_area(cx)),
                        ),
                    )
                    .child(Separator::horizontal())
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Label::new("Keyboard playback mode")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(self.render_playback(cx)),
                    )
                    .child(
                        Label::new(match self.playback_mode {
                            PlaybackMode::Sequential => {
                                "Sounds play in order, cycling back to the first."
                            }
                            PlaybackMode::Random => "Sounds are selected randomly from the pool.",
                        })
                        .text_sm()
                        .text_color(if self.random_playback_available() {
                            cx.theme().foreground
                        } else {
                            cx.theme().muted_foreground
                        }),
                    ),
            ),
        )
    }
}

fn toggle_key_selection(
    selected_keys: &mut [Vec<&'static str>; 3],
    keyboard_layout: KeyboardLayout,
    key_id: &'static str,
    sync_selections: bool,
) {
    if sync_selections {
        let canonical_id = canonical_key_id(key_id);
        for layout in KeyboardLayout::ALL {
            let matching_key = layout
                .key_ids()
                .into_iter()
                .find(|key| canonical_key_id(key) == canonical_id);
            let Some(matching_key) = matching_key else {
                continue;
            };
            let layout_keys = &mut selected_keys[layout.index()];
            if let Some(index) = layout_keys.iter().position(|key| *key == matching_key) {
                layout_keys.remove(index);
            } else {
                layout_keys.push(matching_key);
            }
        }
    } else {
        let layout_keys = &mut selected_keys[keyboard_layout.index()];
        if let Some(index) = layout_keys.iter().position(|key| *key == key_id) {
            layout_keys.remove(index);
        } else {
            layout_keys.push(key_id);
        }
    }
}
