use std::path::PathBuf;

use crate::settings_store::AppSettings;
use crate::storage::{self, LoadedPreset, PresetKind};
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::combobox::{Combobox, ComboboxEvent, ComboboxState};
use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::Label;
use gpui_kit::component::searchable_list::{SearchableListItem, SearchableVec};
use gpui_kit::component::slider::{Slider, SliderEvent, SliderState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::*;
use gpui_kit::*;

#[derive(Clone)]
pub struct PresetChoice {
    pub id: SharedString,
    pub name: SharedString,
    pub summary: SharedString,
    pub data: Option<PresetData>,
    pub is_builtin: bool,
}

#[derive(Clone, Debug)]
pub enum PresetData {
    Keyboard(KeyboardPresetData),
    Mouse(MousePresetData),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardPresetData {
    pub selected_keys: [Vec<&'static str>; 3],
    pub files: Vec<KeyboardSoundData>,
    pub random_playback: bool,
    pub layout_index: usize,
    pub sync_selections: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardSoundData {
    pub path: PathBuf,
    pub name: SharedString,
    pub size: Option<SharedString>,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MousePresetData {
    pub files: Vec<MouseSoundData>,
    pub random_playback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MouseSoundData {
    pub path: PathBuf,
    pub name: SharedString,
    pub size: Option<SharedString>,
    pub assigned_buttons: [bool; 3],
}

impl PresetChoice {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        summary: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            summary: summary.into(),
            data: None,
            is_builtin: true,
        }
    }
}

impl SearchableListItem for PresetChoice {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }

    fn render(&self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex().child(div().child(self.name.clone())).child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(self.summary.clone()),
        )
    }
}

#[derive(Clone, Debug)]
pub enum HomeEvent {
    ImportKeyboardRequested,
    CreateKeyboardRequested,
    EditRequested(SharedString),
    DeleteKeyboardRequested(SharedString),
    ImportMousesRequested,
    CreateMouseRequested,
    ConfigureMouseRequested(SharedString),
    DeleteMouseRequested(SharedString),
    SettingsChanged {
        settings: AppSettings,
        persist: bool,
    },
}

const VOLUME_MIN: f32 = 0.0;
const VOLUME_MAX: f32 = 200.0;

fn new_volume_slider(cx: &mut App, volume: f32) -> Entity<SliderState> {
    cx.new(|_| {
        SliderState::new()
            .min(VOLUME_MIN)
            .max(VOLUME_MAX)
            .step(1.0)
            .default_value(volume)
    })
}

#[allow(clippy::too_many_arguments)]
fn render_summary_card(
    cx: &App,
    icon: IconName,
    button_icon: IconName,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
    button_id: impl Into<ElementId>,
    button_label: impl Into<SharedString>,
    button_disabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let custom_group_home_button = ButtonCustomVariant::new(cx)
        .color(cx.theme().muted_foreground.opacity(0.25))
        .foreground(cx.theme().primary)
        .hover(cx.theme().muted.opacity(0.3))
        .active(cx.theme().muted.opacity(0.5));

    GroupBox::new()
        .fill()
        .border_1()
        .border_color(cx.theme().muted_foreground.opacity(0.3))
        .rounded(cx.theme().radius)
        .child(
            h_flex()
                .items_center()
                .gap_3()
                .child(
                    h_flex()
                        .flex_shrink_0()
                        .w(rems(2.5))
                        .h(rems(2.5))
                        .items_center()
                        .justify_center()
                        .rounded(cx.theme().radius)
                        .bg(cx.theme().blue.opacity(0.15))
                        .child(
                            Icon::new(icon)
                                .with_size(px(30.))
                                .text_color(cx.theme().blue),
                        ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(div().font_semibold().child(title.into()))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(subtitle.into()),
                        ),
                )
                .child(
                    Button::new(button_id)
                        .icon(Icon::new(button_icon).with_size(px(30.)))
                        .border_1()
                        .custom(custom_group_home_button)
                        .label(button_label)
                        .disabled(button_disabled)
                        .on_click(on_click),
                ),
        )
}

fn render_volume_control(
    cx: &App,
    slider: &Entity<SliderState>,
    volume: f32,
    is_muted: bool,
    mute_button_id: impl Into<ElementId>,
    on_toggle_mute: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let icon = if is_muted {
        IconName::VolumeX
    } else {
        IconName::Volume2
    };

    h_flex()
        .items_center()
        .gap_3()
        .child(
            Slider::new(slider)
                .flex_1()
                .text_color(cx.theme().blue)
                .bg(cx.theme().blue),
        )
        .child(
            div()
                .flex_shrink_0()
                .w(rems(3.))
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}%", volume as i32)),
        )
        .child(
            Button::new(mute_button_id)
                .outline()
                .icon(Icon::new(icon).with_size(px(18.)))
                .on_click(on_toggle_mute),
        )
}

fn render_delete_confirmation(
    cx: &mut Context<HomeView>,
    id: SharedString,
    name: SharedString,
    kind: PresetKind,
) -> impl IntoElement {
    let is_keyboard = kind == PresetKind::Keyboard;
    let trigger_id = if is_keyboard {
        "delete-keyboard-preset"
    } else {
        "delete-mouse-preset"
    };
    let entity = cx.entity().downgrade();
    let kind_name = if is_keyboard { "keyboard" } else { "mouse" };
    let description: SharedString = format!(
        "This permanently deletes the {kind_name} preset \"{name}\" and its sound files. This cannot be undone."
    )
    .into();

    AlertDialog::new(cx)
        .trigger(
            Button::new(trigger_id)
                .outline()
                .danger()
                .label("Delete")
                .disabled(id.is_empty()),
        )
        .on_ok(move |_, _, cx| {
            let id = id.clone();
            _ = entity.update(cx, move |_, cx| {
                cx.emit(if is_keyboard {
                    HomeEvent::DeleteKeyboardRequested(id)
                } else {
                    HomeEvent::DeleteMouseRequested(id)
                });
            });
            true
        })
        .content(move |content, _, _| {
            content
                .child(
                    DialogHeader::new()
                        .child(DialogTitle::new().child("Delete preset?"))
                        .child(DialogDescription::new().child(description.clone())),
                )
                .child(
                    DialogFooter::new()
                        .child(
                            DialogClose::new().child(
                                Button::new("cancel-preset-deletion")
                                    .outline()
                                    .label("Cancel"),
                            ),
                        )
                        .child(
                            DialogAction::new().child(
                                Button::new("confirm-preset-deletion")
                                    .outline()
                                    .danger()
                                    .label("Delete preset"),
                            ),
                        ),
                )
        })
}

type PresetItems = SearchableVec<PresetChoice>;

pub struct HomeView {
    presets: Vec<PresetChoice>,
    active_id: SharedString,
    preset_picker: Entity<ComboboxState<PresetItems>>,
    mouse_presets: Vec<PresetChoice>,
    mouse_active_id: SharedString,
    mouse_preset_picker: Entity<ComboboxState<PresetItems>>,
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

impl HomeView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (data_dir, settings, settings_warning) = crate::settings_store::load();
        let (loaded_presets, preset_warnings) = storage::load_presets(&data_dir);
        let mut presets = Vec::new();
        let mut mouse_presets = Vec::new();
        for preset in loaded_presets {
            append_loaded_preset(&mut presets, &mut mouse_presets, preset);
        }
        let mut startup_warnings = preset_warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect::<Vec<_>>();
        if let Some(warning) = settings_warning {
            startup_warnings.push(format!(
                "Settings could not be loaded. Defaults are active: {warning}"
            ));
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

    pub fn active_preset_name(&self) -> SharedString {
        self.active_preset()
            .map(|preset| preset.name.clone())
            .unwrap_or_else(|| SharedString::from("Unknown preset"))
    }

    pub fn active_mouse_preset_name(&self) -> SharedString {
        self.active_mouse_preset()
            .map(|preset| preset.name.clone())
            .unwrap_or_else(|| SharedString::from("Unknown preset"))
    }

    #[allow(clippy::too_many_arguments)]
    fn save_preset(
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

    pub fn keyboard_preset_data(&self, id: &str) -> Option<KeyboardPresetData> {
        self.presets
            .iter()
            .find(|preset| preset.id == id)
            .and_then(|preset| match &preset.data {
                Some(PresetData::Keyboard(data)) => Some(data.clone()),
                _ => None,
            })
    }

    pub fn mouse_preset_data(&self, id: &str) -> Option<MousePresetData> {
        self.mouse_presets
            .iter()
            .find(|preset| preset.id == id)
            .and_then(|preset| match &preset.data {
                Some(PresetData::Mouse(data)) => Some(data.clone()),
                _ => None,
            })
    }

    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    pub fn startup_warnings(&self) -> &[String] {
        &self.startup_warnings
    }

    pub fn is_user_keyboard_preset(&self, id: &str) -> bool {
        self.presets
            .iter()
            .any(|preset| preset.id == id && !preset.is_builtin)
    }

    pub fn is_user_mouse_preset(&self, id: &str) -> bool {
        self.mouse_presets
            .iter()
            .any(|preset| preset.id == id && !preset.is_builtin)
    }

    pub fn settings(&self) -> AppSettings {
        self.settings.clone()
    }

    pub fn set_app_enabled_from_shortcut(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.is_active = enabled;
        self.emit_settings(true, cx);
    }

    pub fn remove_preset(
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

    pub fn apply_saved_keyboard(
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

    pub fn apply_saved_mouse(
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

    fn active_preset(&self) -> Option<&PresetChoice> {
        self.presets
            .iter()
            .find(|preset| preset.id == self.active_id)
    }

    fn active_mouse_preset(&self) -> Option<&PresetChoice> {
        self.mouse_presets
            .iter()
            .find(|preset| preset.id == self.mouse_active_id)
    }

    fn set_active(&mut self, id: SharedString, cx: &mut Context<Self>) {
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

    fn set_mouse_active(&mut self, id: SharedString, cx: &mut Context<Self>) {
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

    fn toggle_mute(
        muted_field: &mut bool,
        _slider: &Entity<SliderState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        *muted_field = !*muted_field;
        cx.notify();
    }

    fn emit_settings(&mut self, persist: bool, cx: &mut Context<Self>) {
        self.settings.keyboard_volume = self.keyboard_volume;
        self.settings.mouse_volume = self.mouse_volume;
        self.settings.keyboard_muted = self.keyboard_muted;
        self.settings.mouse_muted = self.mouse_muted;
        self.settings.app_enabled = self.is_active;
        self.settings.active_keyboard_preset_id =
            (!self.active_id.is_empty()).then(|| self.active_id.to_string());
        self.settings.active_mouse_preset_id =
            (!self.mouse_active_id.is_empty()).then(|| self.mouse_active_id.to_string());
        cx.emit(HomeEvent::SettingsChanged {
            settings: self.settings.clone(),
            persist,
        });
        cx.notify();
    }

    fn render_header(&self) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(Label::new("KeyJolt").text_size(rems(2.2)).font_extrabold())
            .child(
                Label::new(
                    "A simple remapping tool to produce sound feedback for every keystroke and click.",
                )
                .text_lg(),
            )
    }

    fn render_app_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_full()
                            .bg(if self.is_active {
                                cx.theme().green.opacity(0.18)
                            } else {
                                cx.theme().red.opacity(0.18)
                            })
                            .text_xs()
                            .text_color(if self.is_active {
                                cx.theme().green
                            } else {
                                cx.theme().red
                            })
                            .child(if self.is_active { "Active" } else { "Inactive" }),
                    )
                    .child(
                        Label::new("App Status")
                            .text_size(px(17.0))
                            .font_extrabold(),
                    )
                    .child(
                        Label::new("     •     Mute with ")
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        Kbd::new(Keystroke::parse("alt").unwrap())
                            .text_size(px(13.0))
                            .border_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.3)),
                    )
                    .child(
                        Label::new("+")
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        Kbd::new(Keystroke::parse("shift").unwrap())
                            .text_size(px(13.0))
                            .border_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.3)),
                    )
                    .child(
                        Label::new("+")
                            .text_size(px(13.0))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        Kbd::new(Keystroke::parse("m").unwrap())
                            .text_size(px(13.0))
                            .border_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.3)),
                    ),
            )
            .child(
                Switch::new("app-status")
                    .checked(self.is_active)
                    .on_change(cx.listener(|this, checked, _window, cx| {
                        this.is_active = *checked;
                        this.emit_settings(true, cx);
                        cx.notify();
                    })),
            )
    }

    fn render_keyboard_presets(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active_name = self
            .active_preset()
            .map(|preset| preset.name.clone())
            .unwrap_or_else(|| SharedString::from("No preset selected"));

        let summary = self
            .active_preset()
            .map(|preset| preset.summary.clone())
            .unwrap_or_else(|| SharedString::from("Choose a preset above"));

        let has_active_preset = self.active_preset().is_some();
        let active_id = self.active_id.clone();
        let delete_id = active_id.clone();

        v_flex()
            .gap_2()
            .child(
                Label::new("Keyboard Presets")
                    .text_size(rems(0.9375))
                    .text_color(cx.theme().muted_foreground)
                    .font_bold(),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Combobox::new(&self.preset_picker)
                            .placeholder("Select a preset")
                            .cleanable(false)
                            .flex_1(),
                    )
                    .child(render_delete_confirmation(
                        cx,
                        delete_id,
                        active_name.clone(),
                        PresetKind::Keyboard,
                    )),
            )
            .child(render_summary_card(
                cx,
                IconName::Keyboard,
                IconName::Pencil,
                active_name,
                summary,
                "edit-preset",
                "Edit",
                !has_active_preset,
                cx.listener(move |_this, _event, _window, cx| {
                    cx.emit(HomeEvent::EditRequested(active_id.clone()));
                }),
            ))
            .child(render_volume_control(
                cx,
                &self.keyboard_volume_slider,
                self.keyboard_volume,
                self.keyboard_muted,
                "keyboard-mute-toggle",
                cx.listener(|this, _event, window, cx| {
                    let slider = this.keyboard_volume_slider.clone();
                    Self::toggle_mute(&mut this.keyboard_muted, &slider, window, cx);
                    this.emit_settings(true, cx);
                }),
            ))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("import-preset")
                            .outline()
                            .icon(Icon::new(IconName::Import).with_size(px(30.)))
                            .label("Import preset")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(HomeEvent::ImportKeyboardRequested);
                            })),
                    )
                    .child(
                        Button::new("create-preset")
                            .primary()
                            .icon(Icon::new(IconName::Plus).with_size(px(30.)))
                            .label("Create preset")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(HomeEvent::CreateKeyboardRequested);
                            })),
                    ),
            )
    }

    fn render_mouse_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active_name = self
            .active_mouse_preset()
            .map(|preset| preset.name.clone())
            .unwrap_or_else(|| SharedString::from("No preset selected"));
        let summary = self
            .active_mouse_preset()
            .map(|preset| preset.summary.clone())
            .unwrap_or_else(|| SharedString::from("Choose a preset above"));
        let has_active_preset = self.active_mouse_preset().is_some();
        let active_id = self.mouse_active_id.clone();
        let delete_id = active_id.clone();

        v_flex()
            .gap_2()
            .child(
                Label::new("Mouse Presets")
                    .text_size(rems(0.9375))
                    .text_color(cx.theme().muted_foreground)
                    .font_bold(),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Combobox::new(&self.mouse_preset_picker)
                            .placeholder("Select a mouse preset")
                            .cleanable(false)
                            .flex_1(),
                    )
                    .child(render_delete_confirmation(
                        cx,
                        delete_id,
                        active_name.clone(),
                        PresetKind::Mouse,
                    )),
            )
            .child(render_summary_card(
                cx,
                IconName::Mouse,
                IconName::Wrench,
                active_name,
                summary,
                "configure-mouse",
                "Configure",
                !has_active_preset,
                cx.listener(move |_this, _event, _window, cx| {
                    cx.emit(HomeEvent::ConfigureMouseRequested(active_id.clone()));
                }),
            ))
            .child(render_volume_control(
                cx,
                &self.mouse_volume_slider,
                self.mouse_volume,
                self.mouse_muted,
                "mouse-mute-toggle",
                cx.listener(|this, _event, window, cx| {
                    let slider = this.mouse_volume_slider.clone();
                    Self::toggle_mute(&mut this.mouse_muted, &slider, window, cx);
                    this.emit_settings(true, cx);
                }),
            ))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("import-mouse-preset")
                            .outline()
                            .icon(Icon::new(IconName::Import).with_size(px(30.)))
                            .label("Import preset")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(HomeEvent::ImportMousesRequested);
                            })),
                    )
                    .child(
                        Button::new("create-mouse-preset")
                            .primary()
                            .icon(Icon::new(IconName::Plus).with_size(px(30.)))
                            .label("Create preset")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(HomeEvent::CreateMouseRequested);
                            })),
                    ),
            )
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .font_family("Segoe UI")
            .px_8()
            .pt_6()
            .pb_6()
            .gap_12()
            .child(self.render_header())
            .child(self.render_app_status(cx))
            .child(self.render_keyboard_presets(cx))
            .child(self.render_mouse_section(cx))
    }
}

fn append_loaded_preset(
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
