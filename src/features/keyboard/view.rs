use super::KeyboardEditorView;
use crate::features::preset_editor::{PlaybackMode, SaveStatus};

use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::input::Input;
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tab::{Tab, TabBar};

use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl Render for KeyboardEditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_count = self.selected_keys[self.keyboard_layout.index()].len();
        let preset_id = self.preset_id.clone();
        let creating = preset_id.is_none();

        let files = self.files.clone();
        let mut file_rows = Vec::with_capacity(files.len());

        for file in files {
            file_rows.push(self.render_file_row(file, cx).into_any_element());
        }

        let back_control = self.render_back_control(cx);

        let editor_identity = if creating {
            v_flex()
                .gap_1()
                .child(Label::new("Creating preset").text_color(cx.theme().muted_foreground))
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
                                            Label::new("Keyboard editor")
                                                .font_semibold()
                                                .text_size(rems(1.0625)),
                                        )
                                        .child(editor_identity),
                                ),
                            )
                            .child(
                                Button::new("save-preset")
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
                                        .id("keyboard-preset-name")
                                        .cleanable(true)
                                        .aria_label("Preset name"),
                                ),
                        )
                    })
                    .child(Separator::horizontal())
                    .child(
                        TabBar::new("keyboard-layout-tabs")
                            .outline()
                            .small()
                            .selected_index(self.keyboard_layout.index())
                            .on_click(cx.listener(|this, index, _, cx| {
                                this.set_layout(*index, cx);
                            }))
                            .child(Tab::new().label("Full size"))
                            .child(Tab::new().label("TKL"))
                            .child(Tab::new().label("Compact")),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .flex_wrap()
                            .gap_2()
                            .child(Self::render_hint("Click", cx))
                            .child(
                                Label::new("select one key")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(Self::render_kbd_hint("ctrl", cx))
                            .child(Label::new("+").text_sm())
                            .child(Self::render_hint("Click", cx))
                            .child(
                                Label::new("add or remove keys")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .flex_shrink_0()
                                    .child(Self::render_kbd_hint("shift", cx))
                                    .child(Label::new("+").text_sm())
                                    .child(Self::render_hint("Scroll", cx))
                                    .child(
                                        Label::new("horizontal scroll")
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground),
                                    ),
                            ),
                    )
                    .child(self.render_keyboard_area(cx))
                    .child(self.render_assignment_legend(cx))
                    .child(Separator::horizontal())
                    .child(self.render_selection_controls(cx))
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_1()
                                    .child(
                                        Label::new("Sync selections across keyboard sizes")
                                            .font_medium(),
                                    )
                                    .child(
                                        Label::new(
                                            "Sync key selection and sound edits across matching keys. Existing mappings are kept.",
                                        )
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                    ),
                            )
                            .child(
                                Switch::new("sync-keyboard-selections")
                                    .checked(self.sync_selections)
                                    .on_change(cx.listener(|this, checked, _, cx| {
                                        this.set_sync_selections(*checked, cx);
                                    })),
                            ),
                    )
                    .child(Separator::horizontal())
                    .child(
                        h_flex().items_center().justify_between().child(
                            v_flex()
                                .gap_1()
                                .child(
                                    Label::new(format!("Editing {} keys", selected_count))
                                        .font_semibold()
                                        .text_size(rems(1.125)),
                                )
                                .child(
                                    Label::new(format!(
                                        "Assign sounds to this selection · {} keys mapped.",
                                        self.mapped_key_count()
                                    ))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                                ),
                        ),
                    )
                    .child(
                        h_flex().items_start().gap_2().w_full().child(
                            v_flex()
                                .flex_1()
                                .gap_2()
                                .children(file_rows)
                                .child(self.render_upload_area(cx)),
                        ),
                    )
                    .child(Separator::horizontal())
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Label::new("Keyboard playback mode")
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
                            PlaybackMode::Random => "Sounds are selected randomly from the pool.",
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
