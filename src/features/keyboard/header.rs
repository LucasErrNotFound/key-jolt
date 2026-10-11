use super::KeyboardEditorView;
use crate::features::preset_editor::SaveStatus;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .flex_shrink_0()
            .px_4()
            .py_3()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(self.render_back_control(cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(div().font_semibold().text_lg().child("Keyboard editor"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .truncate()
                                    .child(if self.preset_id.is_none() {
                                        "Creating a new preset".into()
                                    } else {
                                        SharedString::from(format!(
                                            "Editing · {}",
                                            self.preset_name
                                        ))
                                    }),
                            ),
                    )
                    .child(
                        Button::new("save-preset")
                            .primary()
                            .label(match self.save_status {
                                SaveStatus::Saving => "Saving…",
                                SaveStatus::Failed(_) => "Retry save",
                                SaveStatus::Idle => "Save preset",
                            })
                            .disabled(
                                matches!(self.save_status, SaveStatus::Saving)
                                    || !self.has_unsaved_changes(cx),
                            )
                            .on_click(
                                cx.listener(|this, _, window, cx| this.request_save(window, cx)),
                            ),
                    ),
            )
            .when(self.preset_id.is_none(), |header| {
                header.child(
                    Input::new(&self.name_input)
                        .id("keyboard-preset-name")
                        .w_full()
                        .cleanable(true)
                        .aria_label("Preset name"),
                )
            })
    }
}
