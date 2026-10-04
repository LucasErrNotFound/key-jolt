use super::{HomeEvent, HomeView};

use super::components::{render_delete_confirmation, render_volume_control};

use super::preset_summary::PresetSummary;

use crate::presets::PresetKind;
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::combobox::Combobox;
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::Label;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::*;
use gpui_kit::*;

impl HomeView {
    pub(super) fn render_header(&self) -> impl IntoElement {
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

    pub(super) fn render_keyboard_presets(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(PresetSummary::new(
                IconName::Keyboard,
                active_name,
                summary,
                Button::new("edit-preset")
                    .icon(Icon::new(IconName::Pencil).with_size(cx.theme().font_size * 1.875))
                    .label("Edit")
                    .disabled(!has_active_preset)
                    .on_click(cx.listener(move |_this, _event, _window, cx| {
                        cx.emit(HomeEvent::EditRequested(active_id.clone()));
                    })),
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
                            .icon(
                                Icon::new(IconName::Import).with_size(cx.theme().font_size * 1.875),
                            )
                            .label("Import preset")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(HomeEvent::ImportKeyboardRequested);
                            })),
                    )
                    .child(
                        Button::new("create-preset")
                            .primary()
                            .icon(Icon::new(IconName::Plus).with_size(cx.theme().font_size * 1.875))
                            .label("Create preset")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(HomeEvent::CreateKeyboardRequested);
                            })),
                    ),
            )
    }

    pub(super) fn render_mouse_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(PresetSummary::new(
                IconName::Mouse,
                active_name,
                summary,
                Button::new("configure-mouse")
                    .icon(Icon::new(IconName::Wrench).with_size(cx.theme().font_size * 1.875))
                    .label("Configure")
                    .disabled(!has_active_preset)
                    .on_click(cx.listener(move |_this, _event, _window, cx| {
                        cx.emit(HomeEvent::ConfigureMouseRequested(active_id.clone()));
                    })),
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
                            .icon(
                                Icon::new(IconName::Import).with_size(cx.theme().font_size * 1.875),
                            )
                            .label("Import preset")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(HomeEvent::ImportMousesRequested);
                            })),
                    )
                    .child(
                        Button::new("create-mouse-preset")
                            .primary()
                            .icon(Icon::new(IconName::Plus).with_size(cx.theme().font_size * 1.875))
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
