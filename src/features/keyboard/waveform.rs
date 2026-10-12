use super::KeyboardEditorView;
use crate::features::preset_editor::render_audio_waveform;
use gpui_kit::component::*;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn render_waveform(&self, id: u64, playing: bool, cx: &App) -> AnyElement {
        render_audio_waveform(
            format!("keyboard-waveform-{id}").into(),
            self.audio_details.get(&id),
            playing,
            cx.theme().primary,
            cx,
        )
    }
}
