use super::assignment::{set_layout_sound_assignment, synchronize_sound_assignments};
use super::layout::KeyboardLayout;
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
    pub(super) assigned_keys: [Vec<String>; 3],
    pub(super) independent_assigned_keys: Option<[Vec<String>; 3]>,
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
            assigned_keys: std::array::from_fn(|_| Vec::new()),
            independent_assigned_keys: None,
            state: UploadState::Uploading,
            preview_state: PreviewState::Stopped,
        }
    }

    pub(super) fn begin_assignment_sync(&mut self, layout: KeyboardLayout) {
        if self.independent_assigned_keys.is_some() {
            return;
        }
        self.independent_assigned_keys = Some(self.assigned_keys.clone());
        synchronize_sound_assignments(&mut self.assigned_keys, layout);
    }

    pub(super) fn end_assignment_sync(&mut self, active_layout: KeyboardLayout) {
        if let Some(mut keys) = self.independent_assigned_keys.take() {
            keys[active_layout.index()] = self.assigned_keys[active_layout.index()].clone();
            self.assigned_keys = keys;
        }
    }

    pub(super) fn edited_assigned_keys(&self) -> &[Vec<String>; 3] {
        self.independent_assigned_keys
            .as_ref()
            .unwrap_or(&self.assigned_keys)
    }

    pub(super) fn set_sound_assignment(
        &mut self,
        selected_keys: &[Vec<&'static str>; 3],
        layout: KeyboardLayout,
        sync: bool,
        enabled: bool,
    ) {
        if sync && self.independent_assigned_keys.is_none() {
            self.independent_assigned_keys = Some(self.assigned_keys.clone());
        }
        if let Some(keys) = &mut self.independent_assigned_keys {
            set_layout_sound_assignment(keys, selected_keys, layout, false, enabled);
        }
        set_layout_sound_assignment(
            &mut self.assigned_keys,
            selected_keys,
            layout,
            sync,
            enabled,
        );
    }
}
