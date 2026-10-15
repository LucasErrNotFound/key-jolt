use super::{HomeEvent, HomeView};
use crate::presets::PresetKind;
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariant, ButtonVariants};
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(super) fn render_delete_confirmation(
    cx: &mut Context<HomeView>,
    id: SharedString,
    name: SharedString,
    kind: PresetKind,
    enabled: bool,
) -> impl IntoElement {
    let is_keyboard = kind == PresetKind::Keyboard;
    let trigger_id = if is_keyboard {
        "delete-keyboard-preset"
    } else {
        "delete-mouse-preset"
    };
    let builtin = crate::presets::is_builtin_preset(kind, &id);
    let entity = cx.entity().downgrade();
    let kind_name = if is_keyboard { "keyboard" } else { "mouse" };
    let description: SharedString = format!(
        "This permanently deletes the {kind_name} preset \"{name}\" and its sound files. This cannot be undone."
    )
    .into();

    Button::new(trigger_id)
        .when(enabled && !id.is_empty() && !builtin, |control| {
            control.cursor_pointer()
        })
        .outline()
        .danger()
        .icon(Icon::new(IconName::Trash).text_color(cx.theme().danger))
        .size_8()
        .flex_shrink_0()
        .accessibility_label("Delete preset")
        .tooltip(if builtin {
            "Built-in presets cannot be deleted; edit one to save a custom copy"
        } else {
            "Delete this preset and its sound files permanently"
        })
        .disabled(!enabled || id.is_empty() || builtin)
        .on_click(cx.listener(move |home, _, window, cx| {
            if !home.is_active || id.is_empty() || builtin {
                return;
            }
            let entity = entity.clone();
            let id = id.clone();
            let description = description.clone();
            window.open_alert_dialog(cx, move |alert, _, _| {
                let entity = entity.clone();
                let id = id.clone();
                alert
                    .title("Delete preset?")
                    .description(description.clone())
                    .confirm()
                    .ok_text("Delete preset")
                    .cancel_text("Cancel")
                    .ok_variant(ButtonVariant::Danger)
                    .on_ok(move |_, _, cx| {
                        let id = id.clone();
                        entity
                            .update(cx, move |home, cx| {
                                if !home.is_active {
                                    return false;
                                }
                                cx.emit(if is_keyboard {
                                    HomeEvent::DeleteKeyboardRequested(id)
                                } else {
                                    HomeEvent::DeleteMouseRequested(id)
                                });
                                true
                            })
                            .unwrap_or(false)
                    })
            });
        }))
}
