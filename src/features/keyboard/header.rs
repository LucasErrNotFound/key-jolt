use super::KeyboardEditorView;
use crate::features::preset_editor::SaveStatus;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mapped = self.mapped_key_count();
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
                                    .id("keyboard-preset-name")
                                    .large()
                                    .w_full()
                                    .h_12()
                                    .py_2()
                                    .text_lg()
                                    .line_height(rems(1.5))
                                    .font_semibold()
                                    .aria_label("Keyboard preset name"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{} keyboard preset · {mapped} keys mapped",
                                        if self.preset_id.is_none() {
                                            "Creating"
                                        } else {
                                            "Editing"
                                        },
                                    )),
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
