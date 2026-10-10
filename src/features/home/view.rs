use super::{HomeEvent, HomeView};

use super::components::render_delete_confirmation;

use super::mixer_strip::MixerStrip;

use crate::app_assets::APP_ICON_PATH;
use crate::presets::PresetKind;
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::combobox::Combobox;
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::Label;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl HomeView {
    pub(super) fn render_header(&self, cx: &App) -> impl IntoElement {
        h_flex()
            .items_center()
            .gap_4()
            .child(
                img(APP_ICON_PATH)
                    .size(rems(4.0))
                    .flex_shrink_0()
                    .object_fit(ObjectFit::Contain),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(Label::new("KeyJolt").text_size(rems(2.2)).font_extrabold())
                    .child(
                        div()
                            .text_base()
                            .text_color(cx.theme().muted_foreground)
                            .child("Custom sound feedback for every keystroke and click."),
                    ),
            )
    }

    pub(super) fn render_app_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .text_size(rems(1.0625))
                            .font_extrabold(),
                    )
                    .child(
                        Label::new("     •     Mute with ")
                            .text_size(rems(0.8125))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        Kbd::new(Keystroke::parse("alt").unwrap())
                            .text_size(rems(0.8125))
                            .border_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.3)),
                    )
                    .child(
                        Label::new("+")
                            .text_size(rems(0.8125))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        Kbd::new(Keystroke::parse("shift").unwrap())
                            .text_size(rems(0.8125))
                            .border_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.3)),
                    )
                    .child(
                        Label::new("+")
                            .text_size(rems(0.8125))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        Kbd::new(Keystroke::parse("m").unwrap())
                            .text_size(rems(0.8125))
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

    pub(super) fn render_mixer_strip(
        &self,
        kind: PresetKind,
        cx: &mut Context<Self>,
    ) -> MixerStrip {
        let keyboard = kind == PresetKind::Keyboard;
        let preset = if keyboard {
            self.active_preset()
        } else {
            self.active_mouse_preset()
        };
        let id = if keyboard {
            self.active_id.clone()
        } else {
            self.mouse_active_id.clone()
        };
        let name = preset
            .map(|preset| preset.name.clone())
            .unwrap_or_else(|| "No preset selected".into());
        let summary = preset
            .map(|preset| {
                format!(
                    "{} · {}",
                    if preset.is_builtin {
                        "Built-in"
                    } else {
                        "Custom"
                    },
                    preset.summary
                )
                .into()
            })
            .unwrap_or_else(|| SharedString::from("Choose or create a preset"));
        let (slider, focus, volume, muted, activity) = if keyboard {
            (
                &self.keyboard_volume_slider,
                &self.keyboard_fader_focus,
                self.keyboard_volume,
                self.keyboard_muted,
                &self.keyboard_activity,
            )
        } else {
            (
                &self.mouse_volume_slider,
                &self.mouse_fader_focus,
                self.mouse_volume,
                self.mouse_muted,
                &self.mouse_activity,
            )
        };
        let picker = if keyboard {
            &self.preset_picker
        } else {
            &self.mouse_preset_picker
        };
        let edit_id = id.clone();
        MixerStrip {
            label: if keyboard { "KEYBOARD" } else { "MOUSE" },
            icon: if keyboard {
                IconName::Keyboard
            } else {
                IconName::Mouse
            },
            picker: Combobox::new(picker)
                .placeholder("Select a preset")
                .cleanable(false)
                .disabled(!self.is_active)
                .flex_1()
                .min_w_0()
                .into_any_element(),
            delete: render_delete_confirmation(cx, id, name, kind, self.is_active)
                .into_any_element(),
            summary,
            slider: slider.clone(),
            focus: focus.clone(),
            volume,
            muted,
            enabled: self.is_active,
            dot: activity.dot.clone(),
            bars: activity.bars.clone(),
            mute: Button::new(if keyboard {
                "keyboard-mute-toggle"
            } else {
                "mouse-mute-toggle"
            })
            .secondary()
            .when(muted, |button| button.danger())
            .disabled(!self.is_active)
            .icon(if muted {
                IconName::VolumeX
            } else {
                IconName::Volume2
            })
            .label(if muted { "Unmute" } else { "Mute" })
            .on_click(cx.listener(move |this, _, window, cx| {
                let slider = if keyboard {
                    this.keyboard_volume_slider.clone()
                } else {
                    this.mouse_volume_slider.clone()
                };
                let muted = if keyboard {
                    &mut this.keyboard_muted
                } else {
                    &mut this.mouse_muted
                };
                Self::toggle_mute(muted, &slider, window, cx);
                this.emit_settings(true, cx);
            })),
            actions: [
                Button::new(if keyboard {
                    "edit-preset"
                } else {
                    "configure-mouse"
                })
                .small()
                .primary()
                .icon(if keyboard {
                    IconName::Pencil
                } else {
                    IconName::Wrench
                })
                .label("Edit")
                .tooltip(if keyboard {
                    "Edit keyboard preset"
                } else {
                    "Configure mouse preset"
                })
                .disabled(!self.is_active || preset.is_none())
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.emit(if keyboard {
                        HomeEvent::EditRequested(edit_id.clone())
                    } else {
                        HomeEvent::ConfigureMouseRequested(edit_id.clone())
                    });
                })),
                Button::new(if keyboard {
                    "import-preset"
                } else {
                    "import-mouse-preset"
                })
                .small()
                .secondary()
                .disabled(!self.is_active)
                .icon(IconName::Import)
                .label("Import")
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.emit(if keyboard {
                        HomeEvent::ImportKeyboardRequested
                    } else {
                        HomeEvent::ImportMousesRequested
                    });
                })),
                Button::new(if keyboard {
                    "create-preset"
                } else {
                    "create-mouse-preset"
                })
                .small()
                .success()
                .disabled(!self.is_active)
                .icon(IconName::Plus)
                .label("New")
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.emit(if keyboard {
                        HomeEvent::CreateKeyboardRequested
                    } else {
                        HomeEvent::CreateMouseRequested
                    });
                })),
            ],
        }
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_6()
            .gap_5()
            .child(self.render_header(cx))
            .child(self.render_app_status(cx))
            .child(
                h_flex()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .gap_4()
                    .when(!self.is_active, |this| {
                        this.opacity(0.32)
                            .capture_key_down(|_, _, cx| cx.stop_propagation())
                    })
                    .child(self.render_mixer_strip(PresetKind::Keyboard, cx))
                    .child(self.render_mixer_strip(PresetKind::Mouse, cx))
                    .when(!self.is_active, |this| {
                        this.child(
                            div()
                                .id("inactive-mixer-blocker")
                                .absolute()
                                .inset_0()
                                .bg(cx.theme().background.opacity(0.5))
                                .occlude()
                                .capture_any_mouse_down(|_, _, cx| cx.stop_propagation())
                                .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation()),
                        )
                    }),
            )
    }
}
