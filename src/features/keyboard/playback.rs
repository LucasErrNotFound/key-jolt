use super::KeyboardEditorView;
use super::assignment::maximum_assigned_sound_count;
use crate::features::preset_editor::PlaybackMode;
use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn set_playback_mode(&mut self, mode: PlaybackMode, cx: &mut Context<Self>) {
        if mode == PlaybackMode::Random && !self.random_playback_available() {
            return;
        }

        self.playback_mode = mode;
        self.preferred_playback_mode = mode;

        cx.notify();
    }

    pub(super) fn random_playback_available(&self) -> bool {
        self.largest_sound_pool_size() >= 2
    }

    pub(super) fn largest_sound_pool_size(&self) -> usize {
        maximum_assigned_sound_count(
            self.files
                .iter()
                .map(|file| file.assigned_keys[self.keyboard_layout.index()].as_slice()),
        )
    }

    pub(super) fn update_playback_mode_for_assignment_change(&mut self, previous_count: usize) {
        let current_count = self.largest_sound_pool_size();
        if current_count == previous_count {
            return;
        }

        if current_count < 2 {
            self.playback_mode = PlaybackMode::Sequential;
        } else if previous_count < 2 {
            self.playback_mode = self.preferred_playback_mode;
        }
    }

    pub(super) fn render_playback(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let random_playback_available = self.random_playback_available();
        let sequential = Button::new("playback-sequential")
            .disabled(!random_playback_available)
            .label("Sequential")
            .when(
                random_playback_available && self.playback_mode == PlaybackMode::Sequential,
                |this| this.primary(),
            )
            .when(
                random_playback_available && self.playback_mode != PlaybackMode::Sequential,
                |this| this.outline(),
            )
            .when(!random_playback_available, |this| {
                this.ghost().text_color(cx.theme().muted_foreground)
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.set_playback_mode(PlaybackMode::Sequential, cx);
            }));

        let random = Button::new("playback-random")
            .disabled(!random_playback_available)
            .label("Random")
            .when(
                random_playback_available && self.playback_mode == PlaybackMode::Random,
                |this| this.primary(),
            )
            .when(
                random_playback_available && self.playback_mode != PlaybackMode::Random,
                |this| this.outline(),
            )
            .when(!random_playback_available, |this| {
                this.ghost().text_color(cx.theme().muted_foreground)
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.set_playback_mode(PlaybackMode::Random, cx);
            }));

        h_flex().gap_1().child(sequential).child(random)
    }
}
