mod activity;
mod activity_meter;
mod components;
mod events;
mod lifecycle;
mod meter_state;
mod mixer_strip;
mod preset_choice;
mod presets;
mod settings;
mod startup;
mod view;
mod volume_fader;

use crate::presets::LoadedPreset;
use crate::settings::store::AppSettings;
use gpui_kit::component::combobox::ComboboxState;
use gpui_kit::component::slider::SliderState;
use gpui_kit::*;
use std::path::PathBuf;

pub(crate) use events::HomeEvent;
use preset_choice::{PresetChoice, PresetItems};

pub(crate) struct HomeView {
    presets: Vec<PresetChoice>,
    active_id: SharedString,
    preset_picker: Entity<ComboboxState<PresetItems>>,
    mouse_presets: Vec<PresetChoice>,
    mouse_active_id: SharedString,
    mouse_preset_picker: Entity<ComboboxState<PresetItems>>,
    keyboard_activity: activity_meter::ActivityWidgets,
    mouse_activity: activity_meter::ActivityWidgets,
    keyboard_fader_focus: FocusHandle,
    mouse_fader_focus: FocusHandle,
    mixer_visible: bool,
    activity_task: Option<Task<()>>,
    keyboard_volume: f32,
    keyboard_volume_slider: Entity<SliderState>,
    mouse_volume: f32,
    mouse_volume_slider: Entity<SliderState>,
    is_active: bool,
    keyboard_muted: bool,
    mouse_muted: bool,
    data_dir: PathBuf,
    startup_warnings: Vec<String>,
    settings: AppSettings,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<HomeEvent> for HomeView {}

pub(crate) struct HomeStartup {
    data_dir: PathBuf,
    settings: AppSettings,
    loaded_presets: Vec<LoadedPreset>,
    startup_warnings: Vec<String>,
}
