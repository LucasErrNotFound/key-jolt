use super::MouseEditorView;
use super::model::MouseButton;
use crate::features::preset_editor::{PlaybackMode, SaveStatus};

use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::input::Input;
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl Render for MouseEditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button = self.selected_button;
        let files = self.files.clone();
        let selected_label = button.label();
        let file_count = self
            .files
            .iter()
            .filter(|file| file.assigned_buttons[button.index()])
            .count();
        let preset_id = self.preset_id.clone();
        let creating = preset_id.is_none();
        let mut file_rows = Vec::with_capacity(files.len());
        for file in files {
            file_rows.push(self.render_file_row(file, button, cx).into_any_element());
        }
        let back_control = self.render_back_control(cx);

        let editor_identity = if creating {
            v_flex()
                .gap_1()
                .child(Label::new("Creating mouse preset").text_color(cx.theme().muted_foreground))
                .into_any_element()
        } else {
            h_flex()
                .gap_1()
                .child(Label::new("Editing").text_color(cx.theme().muted_foreground))
                .child(Label::new(self.preset_name.clone()))
                .into_any_element()
        };

        v_flex().size_full().child(
            div().flex_1().min_h_0().overflow_y_scrollbar().child(
                v_flex()
                    .px_6()
                    .py_4()
                    .gap_4()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .child(
                                h_flex().items_center().gap_3().child(back_control).child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            Label::new("Mouse configuration")
                                                .font_semibold()
                                                .text_size(rems(1.0625)),
                                        )
                                        .child(editor_identity),
                                ),
                            )
                            .child(
                                Button::new("save-mouse-preset")
                                    .primary()
                                    .label(match &self.save_status {
                                        SaveStatus::Idle => "Save preset",
                                        SaveStatus::Saving => "Saving…",
                                        SaveStatus::Failed(_) => "Retry save",
                                    })
                                    .disabled(
                                        matches!(self.save_status, SaveStatus::Saving)
                                            || !self.has_unsaved_changes(cx),
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.request_save(window, cx)
                                    })),
                            ),
                    )
                    .when(creating, |this| {
                        this.child(
                            v_flex()
                                .w_full()
                                .gap_1()
                                .child(Label::new("Preset name").text_sm())
                                .child(
                                    Input::new(&self.name_input)
                                        .w(rems(17.5))
                                        .id("mouse-preset-name")
                                        .cleanable(true)
                                        .aria_label("Preset name"),
                                ),
                        )
                    })
                    .child(Separator::horizontal())
                    .child(
                        h_flex()
                            .items_center()
                            .justify_center()
                            .gap_8()
                            .py_2()
                            .child(self.render_mouse_diagram(cx))
                            .child(
                                v_flex()
                                    .w(rems(13.75))
                                    .gap_2()
                                    .child(self.render_mouse_button(
                                        MouseButton::Left,
                                        "Left click",
                                        cx,
                                    ))
                                    .child(self.render_mouse_button(
                                        MouseButton::Right,
                                        "Right click",
                                        cx,
                                    ))
                                    .child(self.render_mouse_button(
                                        MouseButton::Middle,
                                        "Middle · scroll",
                                        cx,
                                    )),
                            ),
                    )
                    .child(Separator::horizontal())
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                Label::new(format!("Editing {selected_label}"))
                                    .font_semibold()
                                    .text_size(rems(1.125)),
                            )
                            .child(
                                Label::new(if file_count == 0 {
                                    "Assign one sound, or several for a random pool.".to_string()
                                } else {
                                    format!(
                                        "{file_count} sound{} assigned to {selected_label}.",
                                        if file_count == 1 { "" } else { "s" }
                                    )
                                })
                                .text_sm()
                                .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .children(file_rows)
                            .child(self.render_upload_area(cx)),
                    )
                    .child(Separator::horizontal())
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Label::new("Playback")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(self.render_playback(cx)),
                    )
                    .child(
                        Label::new(match self.playback_mode {
                            PlaybackMode::Sequential => {
                                "Sounds play in order, cycling back to the first."
                            }
                            PlaybackMode::Random => {
                                "Sounds are selected randomly from this button’s pool."
                            }
                        })
                        .text_sm()
                        .text_color(if self.random_playback_available() {
                            cx.theme().foreground
                        } else {
                            cx.theme().muted_foreground
                        }),
                    ),
            ),
        )
    }
}
