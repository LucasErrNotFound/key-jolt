use super::{HomeEvent, HomeView};

use super::preset_choice::PresetChoice;
use crate::presets::{KeyboardPresetData, LoadedPreset, MousePresetData, PresetData, PresetKind};

use gpui_kit::component::searchable_list::SearchableVec;
use gpui_kit::*;

impl HomeView {
    pub(crate) fn active_preset_name(&self) -> SharedString {
        self.active_preset()
            .map(|preset| preset.name.clone())
            .unwrap_or_else(|| SharedString::from("Unknown preset"))
    }

    pub(crate) fn active_mouse_preset_name(&self) -> SharedString {
        self.active_mouse_preset()
            .map(|preset| preset.name.clone())
            .unwrap_or_else(|| SharedString::from("Unknown preset"))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn save_preset(
        &mut self,
        preset_id: Option<SharedString>,
        name: SharedString,
        summary: SharedString,
        data: PresetData,
        is_mouse: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SharedString {
        let presets = if is_mouse {
            &mut self.mouse_presets
        } else {
            &mut self.presets
        };

        let id = preset_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string().into());

        if let Some(preset) = presets.iter_mut().find(|preset| preset.id == id) {
            preset.name = name;
            preset.summary = summary;
            preset.data = Some(data);
            preset.is_builtin = false;
        } else {
            let mut preset = PresetChoice::new(id.clone(), name, summary);
            preset.data = Some(data);
            preset.is_builtin = false;
            presets.push(preset);
        }

        if is_mouse {
            self.mouse_active_id = id.clone();
            self.settings.active_mouse_preset_id = Some(id.to_string());
            let items = self.mouse_presets.clone();
            self.mouse_preset_picker.update(cx, |picker, cx| {
                picker.set_items(SearchableVec::new(items), window, cx);
                picker.set_selected_values(std::slice::from_ref(&id), window, cx);
            });
        } else {
            self.active_id = id.clone();
            self.settings.active_keyboard_preset_id = Some(id.to_string());
            let items = self.presets.clone();
            self.preset_picker.update(cx, |picker, cx| {
                picker.set_items(SearchableVec::new(items), window, cx);
                picker.set_selected_values(std::slice::from_ref(&id), window, cx);
            });
        }
        self.emit_settings(true, cx);
        cx.notify();
        id
    }

    pub(crate) fn keyboard_preset_data(&self, id: &str) -> Option<KeyboardPresetData> {
        self.presets
            .iter()
            .find(|preset| preset.id == id)
            .and_then(|preset| match &preset.data {
                Some(PresetData::Keyboard(data)) => Some(data.clone()),
                _ => None,
            })
    }

    pub(crate) fn mouse_preset_data(&self, id: &str) -> Option<MousePresetData> {
        self.mouse_presets
            .iter()
            .find(|preset| preset.id == id)
            .and_then(|preset| match &preset.data {
                Some(PresetData::Mouse(data)) => Some(data.clone()),
                _ => None,
            })
    }

    pub(crate) fn is_user_keyboard_preset(&self, id: &str) -> bool {
        self.presets
            .iter()
            .any(|preset| preset.id == id && !preset.is_builtin)
    }

    pub(crate) fn is_user_mouse_preset(&self, id: &str) -> bool {
        self.mouse_presets
            .iter()
            .any(|preset| preset.id == id && !preset.is_builtin)
    }

    pub(crate) fn remove_preset(
        &mut self,
        kind: PresetKind,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match kind {
            PresetKind::Keyboard => {
                self.presets.retain(|preset| preset.id != id);
                if self.active_id.as_ref() == id {
                    self.active_id = self
                        .presets
                        .first()
                        .map(|preset| preset.id.clone())
                        .unwrap_or_else(|| SharedString::from(""));
                }
                let items = self.presets.clone();
                let selected = self.active_id.clone();
                self.preset_picker.update(cx, |picker, cx| {
                    picker.set_items(SearchableVec::new(items), window, cx);
                    if selected.is_empty() {
                        picker.set_selected_values(&[], window, cx);
                    } else {
                        picker.set_selected_values(std::slice::from_ref(&selected), window, cx);
                    }
                });
                self.settings.active_keyboard_preset_id =
                    (!self.active_id.is_empty()).then(|| self.active_id.to_string());
            }
            PresetKind::Mouse => {
                self.mouse_presets.retain(|preset| preset.id != id);
                if self.mouse_active_id.as_ref() == id {
                    self.mouse_active_id = self
                        .mouse_presets
                        .first()
                        .map(|preset| preset.id.clone())
                        .unwrap_or_else(|| SharedString::from(""));
                }
                let items = self.mouse_presets.clone();
                let selected = self.mouse_active_id.clone();
                self.mouse_preset_picker.update(cx, |picker, cx| {
                    picker.set_items(SearchableVec::new(items), window, cx);
                    if selected.is_empty() {
                        picker.set_selected_values(&[], window, cx);
                    } else {
                        picker.set_selected_values(std::slice::from_ref(&selected), window, cx);
                    }
                });
                self.settings.active_mouse_preset_id =
                    (!self.mouse_active_id.is_empty()).then(|| self.mouse_active_id.to_string());
            }
        }
        self.emit_settings(true, cx);
    }

    pub(crate) fn apply_saved_keyboard(
        &mut self,
        id: SharedString,
        name: SharedString,
        summary: SharedString,
        data: KeyboardPresetData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_preset(
            Some(id),
            name,
            summary,
            PresetData::Keyboard(data),
            false,
            window,
            cx,
        );
    }

    pub(crate) fn apply_saved_mouse(
        &mut self,
        id: SharedString,
        name: SharedString,
        summary: SharedString,
        data: MousePresetData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_preset(
            Some(id),
            name,
            summary,
            PresetData::Mouse(data),
            true,
            window,
            cx,
        );
    }

    pub(super) fn active_preset(&self) -> Option<&PresetChoice> {
        self.presets
            .iter()
            .find(|preset| preset.id == self.active_id)
    }

    pub(super) fn active_mouse_preset(&self) -> Option<&PresetChoice> {
        self.mouse_presets
            .iter()
            .find(|preset| preset.id == self.mouse_active_id)
    }

    pub(super) fn set_active(&mut self, id: SharedString, cx: &mut Context<Self>) {
        if self.active_id != id {
            self.active_id = id;
            self.settings.active_keyboard_preset_id = Some(self.active_id.to_string());
            cx.emit(HomeEvent::SettingsChanged {
                settings: self.settings.clone(),
                persist: true,
            });
            cx.notify();
        }
    }

    pub(super) fn set_mouse_active(&mut self, id: SharedString, cx: &mut Context<Self>) {
        if self.mouse_active_id != id {
            self.mouse_active_id = id;
            self.settings.active_mouse_preset_id = Some(self.mouse_active_id.to_string());
            cx.emit(HomeEvent::SettingsChanged {
                settings: self.settings.clone(),
                persist: true,
            });
            cx.notify();
        }
    }
}

pub(super) fn append_loaded_preset(
    keyboard_presets: &mut Vec<PresetChoice>,
    mouse_presets: &mut Vec<PresetChoice>,
    loaded: LoadedPreset,
) {
    match loaded.kind {
        PresetKind::Keyboard => {
            if let Some(data) = loaded.keyboard {
                let mut preset = PresetChoice::new(loaded.id, loaded.name, loaded.summary);
                preset.data = Some(PresetData::Keyboard(data));
                preset.is_builtin = false;
                keyboard_presets.push(preset);
            }
        }
        PresetKind::Mouse => {
            if let Some(data) = loaded.mouse {
                let mut preset = PresetChoice::new(loaded.id, loaded.name, loaded.summary);
                preset.data = Some(PresetData::Mouse(data));
                preset.is_builtin = false;
                mouse_presets.push(preset);
            }
        }
    }
}
