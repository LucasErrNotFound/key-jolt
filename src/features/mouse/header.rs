use super::MouseEditorView;
use crate::features::preset_editor::{SaveStatus, UploadState};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let save_disabled =
            matches!(self.save_status, SaveStatus::Saving) || !self.has_unsaved_changes(cx);
        let mapped = (0..3)
            .filter(|index| {
                self.files.iter().any(|file| {
                    file.assigned_buttons[*index]
                        && matches!(file.state, UploadState::Success | UploadState::Complete)
                })
            })
            .count();
        v_flex()
            .flex_shrink_0()
            .px_4()
            .py_3()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(self.render_back_control(cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(
                                Input::new(&self.name_input)
                                    .id("mouse-preset-name")
                                    .w_full()
                                    .text_lg()
                                    .font_semibold()
                                    .aria_label("Mouse preset name"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{} mouse preset · {mapped} of 3 buttons have sounds",
                                        if self.preset_id.is_none() {
                                            "Creating"
                                        } else {
                                            "Editing"
                                        },
                                    )),
                            ),
                    )
                    .child(
                        Button::new("save-mouse-preset")
                            .when(!save_disabled, |control| control.cursor_pointer())
                            .primary()
                            .label(match self.save_status {
                                SaveStatus::Saving => "Saving…",
                                SaveStatus::Failed(_) => "Retry save",
                                SaveStatus::Idle => "Save preset",
                            })
                            .disabled(save_disabled)
                            .tooltip(match self.save_status {
                                SaveStatus::Saving => "Saving preset…",
                                SaveStatus::Failed(_) => "Retry saving this preset",
                                SaveStatus::Idle if save_disabled => "No preset changes to save",
                                SaveStatus::Idle => "Save this preset and return to home",
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.request_save(window, cx);
                            })),
                    ),
            )
            .when(
                matches!(self.save_status, SaveStatus::Failed(_)),
                |header| {
                    let message = match &self.save_status {
                        SaveStatus::Failed(message) => message.clone(),
                        _ => SharedString::default(),
                    };
                    header.child(div().text_xs().text_color(cx.theme().danger).child(message))
                },
            )
    }
}
