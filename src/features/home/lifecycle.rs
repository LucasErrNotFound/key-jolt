use super::{HomeStartup, HomeView};

use super::components::new_volume_slider;
use super::preset_choice::PresetItems;
use super::presets::append_loaded_preset;
use gpui_kit::component::combobox::{ComboboxEvent, ComboboxState};

use gpui_kit::component::searchable_list::SearchableVec;
use gpui_kit::component::slider::SliderEvent;
use gpui_kit::component::*;
use gpui_kit::*;

impl HomeView {
    pub(crate) fn view(startup: HomeStartup, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(startup, window, cx))
    }

    pub(super) fn new(startup: HomeStartup, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let HomeStartup {
            data_dir,
            settings,
            loaded_presets,
            startup_warnings,
        } = startup;
        let mut presets = Vec::new();
        let mut mouse_presets = Vec::new();
        for preset in loaded_presets {
            append_loaded_preset(&mut presets, &mut mouse_presets, preset);
        }
        let active_id = settings
            .active_keyboard_preset_id
            .as_deref()
            .and_then(|id| presets.iter().find(|preset| preset.id == id))
            .map(|preset| preset.id.clone())
            .unwrap_or_else(|| SharedString::from(""));
        let mouse_active_id = settings
            .active_mouse_preset_id
            .as_deref()
            .and_then(|id| mouse_presets.iter().find(|preset| preset.id == id))
            .map(|preset| preset.id.clone())
            .unwrap_or_else(|| SharedString::from(""));
        let keyboard_active_index = presets
            .iter()
            .position(|preset| preset.id == active_id)
            .unwrap_or(0);
        let mouse_active_index = mouse_presets
            .iter()
            .position(|preset| preset.id == mouse_active_id)
            .unwrap_or(0);

        let preset_picker = cx.new(|cx| {
            ComboboxState::new(
                SearchableVec::new(presets.clone()),
                if presets.is_empty() {
                    Vec::new()
                } else {
                    vec![IndexPath::default().row(keyboard_active_index)]
                },
                window,
                cx,
            )
        });

        let preset_subscription = cx.subscribe_in(
            &preset_picker,
            window,
            |this, _picker, event: &ComboboxEvent<PresetItems>, _window, cx| {
                if let ComboboxEvent::Confirm(values) = event
                    && let Some(id) = values.first()
                {
                    this.set_active(id.clone(), cx);
                }
            },
        );

        let mouse_preset_picker = cx.new(|cx| {
            ComboboxState::new(
                SearchableVec::new(mouse_presets.clone()),
                if mouse_presets.is_empty() {
                    Vec::new()
                } else {
                    vec![IndexPath::default().row(mouse_active_index)]
                },
                window,
                cx,
            )
        });

        let mouse_preset_subscription = cx.subscribe_in(
            &mouse_preset_picker,
            window,
            |this, _picker, event: &ComboboxEvent<PresetItems>, _window, cx| {
                if let ComboboxEvent::Confirm(values) = event
                    && let Some(id) = values.first()
                {
                    this.set_mouse_active(id.clone(), cx);
                }
            },
        );

        let keyboard_volume_slider = new_volume_slider(cx, settings.keyboard_volume);
        let keyboard_volume_subscription = cx.subscribe_in(
            &keyboard_volume_slider,
            window,
            |this, _slider, event: &SliderEvent, _window, cx| match event {
                SliderEvent::Change(value) => {
                    this.keyboard_volume = value.start();
                    this.keyboard_muted = false;
                    this.emit_settings(false, cx);
                }
                SliderEvent::Release(value) => {
                    this.keyboard_volume = value.start();
                    this.keyboard_muted = false;
                    this.emit_settings(true, cx);
                }
            },
        );

        let mouse_volume_slider = new_volume_slider(cx, settings.mouse_volume);
        let mouse_volume_subscription = cx.subscribe_in(
            &mouse_volume_slider,
            window,
            |this, _slider, event: &SliderEvent, _window, cx| match event {
                SliderEvent::Change(value) => {
                    this.mouse_volume = value.start();
                    this.mouse_muted = false;
                    this.emit_settings(false, cx);
                }
                SliderEvent::Release(value) => {
                    this.mouse_volume = value.start();
                    this.mouse_muted = false;
                    this.emit_settings(true, cx);
                }
            },
        );

        Self {
            presets,
            active_id,
            preset_picker,
            mouse_presets,
            mouse_active_id,
            mouse_preset_picker,
            keyboard_volume: settings.keyboard_volume,
            keyboard_volume_slider,
            mouse_volume: settings.mouse_volume,
            mouse_volume_slider,
            is_active: settings.app_enabled,
            keyboard_muted: settings.keyboard_muted,
            mouse_muted: settings.mouse_muted,
            data_dir,
            startup_warnings,
            settings,
            _subscriptions: vec![
                preset_subscription,
                mouse_preset_subscription,
                keyboard_volume_subscription,
                mouse_volume_subscription,
            ],
        }
    }
}
