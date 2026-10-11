use super::KeyboardEditorView;
use super::assignment::assignment_groups;
use crate::presets::canonical_key_identifier;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(super) struct SoundGroup {
    pub(super) index: usize,
    pub(super) keys: Vec<&'static str>,
    pub(super) sounds: Vec<u64>,
}

impl KeyboardEditorView {
    pub(super) fn sound_groups(&self) -> Vec<SoundGroup> {
        let layout = self.keyboard_layout;
        let groups = assignment_groups(
            self.files
                .iter()
                .map(|file| (file.id, file.assigned_keys[layout.index()].as_slice())),
        );
        groups
            .sounds
            .into_iter()
            .enumerate()
            .map(|(index, sounds)| {
                let keys = layout
                    .key_ids()
                    .into_iter()
                    .filter(|key| groups.keys.get(canonical_key_identifier(key)) == Some(&index))
                    .collect();
                SoundGroup {
                    index,
                    keys,
                    sounds,
                }
            })
            .collect()
    }

    pub(super) fn render_sound_groups(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let groups = self.sound_groups();
        v_flex()
            .gap_2()
            .child(div().font_medium().text_sm().child("Sound groups"))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Choose a group to edit its keys. Colors identify matching sound sets."),
            )
            .when(groups.is_empty(), |list| {
                list.child(
                    div()
                        .py_3()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No mapped keys yet. Select keys above to assign sounds."),
                )
            })
            .children(groups.into_iter().map(|group| {
                let color = Self::assignment_color(group.index, cx);
                let names = self.assignment_sound_names(&group.sounds);
                let count = group.keys.len();
                Button::new(format!("sound-group-{}", group.index))
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .color(color.opacity(0.08))
                            .foreground(cx.theme().foreground)
                            .hover(color.opacity(0.14))
                            .active(color.opacity(0.2)),
                    )
                    .w_full()
                    .h_auto()
                    .py_3()
                    .px_3()
                    .border_1()
                    .border_color(color.opacity(0.35))
                    .accessibility_label(format!(
                        "Group {} · {} keys · {}",
                        group.index + 1,
                        count,
                        names
                    ))
                    .tooltip(names.clone())
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .child(
                                div()
                                    .size(rems(0.625))
                                    .rounded_full()
                                    .bg(color)
                                    .flex_shrink_0(),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_1()
                                    .child(
                                        h_flex()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .font_medium()
                                                    .child(format!("Group {}", group.index + 1)),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!("{count} keys")),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .truncate()
                                            .child(names),
                                    ),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.replace_editing_selection(&group.keys, cx);
                    }))
            }))
    }
}
