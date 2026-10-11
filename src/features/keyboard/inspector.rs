use super::KeyboardEditorView;
use crate::presets::canonical_key_identifier;
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn render_inspector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = &self.selected_keys[self.keyboard_layout.index()];
        let rows = self
            .files
            .iter()
            .filter(|file| {
                selected.is_empty()
                    || self.show_sound_catalog
                    || selected.iter().any(|key| {
                        file.assigned_keys[self.keyboard_layout.index()]
                            .iter()
                            .any(|assigned| {
                                canonical_key_identifier(assigned) == canonical_key_identifier(key)
                            })
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        let has_rows = !rows.is_empty();
        v_flex()
            .size_full()
            .min_h_0()
            .child(
                v_flex()
                    .flex_shrink_0()
                    .px_4()
                    .py_3()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(div().font_semibold().child(if selected.is_empty() {
                                "Preset sounds".to_string()
                            } else {
                                format!(
                                    "{} key{} selected",
                                    selected.len(),
                                    if selected.len() == 1 { "" } else { "s" },
                                )
                            }))
                            .when(!selected.is_empty(), |header| {
                                header.child(
                                    Button::new("inspector-clear-selection")
                                        .ghost()
                                        .small()
                                        .label("Clear")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.clear_selected_keys(cx);
                                        })),
                                )
                            }),
                    ),
            )
            .child(Separator::horizontal())
            .child(
                div()
                    .id("keyboard-inspector-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(
                        v_flex()
                            .p_4()
                            .gap_3()
                            .when(selected.is_empty(), |body| {
                                body.child(self.render_sound_groups(cx))
                                    .child(Separator::horizontal())
                            })
                            .when(!selected.is_empty() && !has_rows, |body| {
                                body.child(
                                    v_flex()
                                        .py_3()
                                        .gap_1()
                                        .child(
                                            div()
                                                .font_medium()
                                                .text_sm()
                                                .child("No sounds assigned to these keys"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Choose an existing sound or upload your own audio."),
                                        ),
                                )
                            })
                            .children(rows.into_iter().map(|file| {
                                self.render_file_row(file, cx).into_any_element()
                            }))
                            .when(!selected.is_empty() && !self.files.is_empty(), |body| {
                                body.child(
                                    Button::new("assign-existing-sound")
                                        .outline()
                                        .small()
                                        .icon(IconName::Plus)
                                        .label(if self.show_sound_catalog {
                                            "Show assigned sounds"
                                        } else {
                                            "Assign existing sound…"
                                        })
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.show_sound_catalog = !this.show_sound_catalog;
                                            cx.notify();
                                        })),
                                )
                            })
                            .child(self.render_upload_area(cx))
                            .child(Separator::horizontal())
                            .child(
                                div()
                                    .font_medium()
                                    .text_sm()
                                    .child("Playback mode · entire preset"),
                            )
                            .child(self.render_playback(cx))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Random is available when a key in this keyboard size has two or more sounds."),
                            ),
                    ),
            )
    }

    pub(super) fn render_sync_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_shrink_0()
            .px_4()
            .py_3()
            .gap_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .child("Sync across keyboard sizes"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Apply sound edits to matching keys in other layouts."),
                    ),
            )
            .child(
                Switch::new("sync-keyboard-selections")
                    .checked(self.sync_selections)
                    .on_change(
                        cx.listener(|this, checked, _, cx| this.set_sync_selections(*checked, cx)),
                    ),
            )
    }

}
