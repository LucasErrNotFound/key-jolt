use super::KeyboardEditorView;
use super::audio_details::AudioDetails;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn render_waveform(&self, id: u64, playing: bool, cx: &App) -> AnyElement {
        match self.audio_details.get(&id) {
            Some(AudioDetails::Ready {
                duration,
                peaks,
                prefix,
            }) => {
                let duration = duration
                    .map(|value| format!("{:.2} s", value.as_secs_f64()))
                    .unwrap_or_else(|| "Duration unknown".into());
                let description: SharedString = if *prefix {
                    "Waveform of the analyzed beginning of this file".into()
                } else {
                    "Waveform of this audio file".into()
                };
                h_flex()
                    .w_full()
                    .gap_3()
                    .child(
                        h_flex()
                            .id(format!("waveform-{id}"))
                            .flex_1()
                            .min_w_0()
                            .h(rems(1.5))
                            .gap(px(1.))
                            .items_center()
                            .aria_label(description.clone())
                            .tooltip(move |window, cx| {
                                Tooltip::new(description.clone()).build(window, cx)
                            })
                            .children(peaks.iter().map(|peak| {
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .h(rems((peak * 1.25).max(0.0625)))
                                    .rounded_full()
                                    .bg(if playing {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().muted_foreground.opacity(0.55)
                                    })
                            })),
                    )
                    .child(
                        v_flex()
                            .flex_shrink_0()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(duration)
                            .when(*prefix, |label| label.child("Partial waveform")),
                    )
                    .into_any_element()
            }
            Some(AudioDetails::Unavailable(message)) => div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            _ => div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Analyzing audio…")
                .into_any_element(),
        }
    }
}
