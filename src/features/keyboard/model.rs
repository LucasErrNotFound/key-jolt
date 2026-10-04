use crate::features::preset_editor::{PreviewState, UploadState};

use gpui_kit::*;
use std::path::PathBuf;

pub(super) struct SelectionGroup {
    pub(super) id: &'static str,
    pub(super) label: &'static str,
    pub(super) keys: Vec<&'static str>,
}

#[derive(Clone)]
pub(super) struct AudioFile {
    pub(super) id: u64,
    pub(super) path: PathBuf,
    pub(super) name: SharedString,
    pub(super) size: Option<SharedString>,
    pub(super) selected: bool,
    pub(super) state: UploadState,
    pub(super) preview_state: PreviewState,
}

impl AudioFile {
    pub(super) fn new(id: u64, path: PathBuf, name: impl Into<SharedString>) -> Self {
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
