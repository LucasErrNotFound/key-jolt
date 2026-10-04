use super::{HomeEvent, HomeView};

use crate::presets::PresetKind;
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};

use gpui_kit::component::slider::{Slider, SliderState};

use gpui_kit::component::*;
use gpui_kit::*;

const VOLUME_MIN: f32 = 0.0;

const VOLUME_MAX: f32 = 200.0;

pub(super) fn new_volume_slider(cx: &mut App, volume: f32) -> Entity<SliderState> {
    cx.new(|_| {
        SliderState::new()
            .min(VOLUME_MIN)
            .max(VOLUME_MAX)
            .step(1.0)
            .default_value(volume)
    })
}

pub(super) fn render_volume_control(
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
                .text_color(cx.theme().primary)
                .bg(cx.theme().primary),
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
                .icon(Icon::new(icon).with_size(cx.theme().font_size * 1.125))
                .on_click(on_toggle_mute),
        )
}

pub(super) fn render_delete_confirmation(
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
