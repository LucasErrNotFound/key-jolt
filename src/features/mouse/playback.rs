use super::MouseEditorView;
use crate::features::preset_editor::{PlaybackMode, PlaybackSelector};
use gpui_kit::component::*;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn set_playback_mode(&mut self, mode: PlaybackMode, cx: &mut Context<Self>) {
        if mode == PlaybackMode::Random && !self.random_playback_available() {
            return;
        }
        self.playback_mode = mode;
        cx.notify();
    }

    pub(super) fn random_playback_available(&self) -> bool {
        self.assigned_sound_count() >= 2
    }

    pub(super) fn assigned_sound_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.assigned_buttons[self.selected_button.index()])
            .count()
    }

    pub(super) fn update_playback_mode_for_selection_change(&mut self, previous_count: usize) {
        let current_count = self.assigned_sound_count();
        if current_count == previous_count {
            return;
        }
        if current_count < 2 || previous_count < 2 {
            self.playback_mode = PlaybackMode::Sequential;
        }
    }

    pub(super) fn render_playback(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        PlaybackSelector::new(
                            "mouse-playback-mode",
                            self.playback_mode,
                            self.random_playback_available(),
                        )
                        .on_change(cx.listener(|this, index, _, cx| {
                            this.set_playback_mode(
                                if *index == 1 {
                                    PlaybackMode::Random
                                } else {
                                    PlaybackMode::Sequential
                                },
                                cx,
                            );
                        })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(if !self.random_playback_available() {
                                "Assign at least two sounds to enable random playback."
                            } else if self.playback_mode == PlaybackMode::Random {
                                "Chooses a random sound on each press."
                            } else {
                                "Plays the assigned sounds in order."
                            }),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Playback mode applies to the entire mouse preset."),
            )
    }
}
