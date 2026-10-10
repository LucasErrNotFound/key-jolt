use super::activity_meter::ActivityIndicator;
use super::volume_fader::{FADER_HEIGHT, VolumeFader, render_fader_scale};
use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::slider::SliderState;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

#[derive(IntoElement)]
pub(super) struct MixerStrip {
    pub(super) label: &'static str,
    pub(super) icon: IconName,
    pub(super) picker: AnyElement,
    pub(super) delete: AnyElement,
    pub(super) summary: SharedString,
    pub(super) slider: Entity<SliderState>,
    pub(super) focus: FocusHandle,
    pub(super) volume: f32,
    pub(super) muted: bool,
    pub(super) enabled: bool,
    pub(super) dot: Entity<ActivityIndicator>,
    pub(super) bars: Entity<ActivityIndicator>,
    pub(super) mute: Button,
    pub(super) actions: [Button; 3],
}

impl RenderOnce for MixerStrip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .p_4()
            .gap_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary.opacity(0.4))
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Icon::new(self.icon).with_size(window.rem_size() * 0.9))
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(self.label),
                            ),
                    )
                    .child(self.dot),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .items_center()
                    .child(self.picker)
                    .child(self.delete),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.summary),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .items_center()
                    .justify_center()
                    .gap_5()
                    .child(
                        h_flex()
                            .id(format!("mixer-fader-{}", self.label))
                            .items_center()
                            .gap_4()
                            .tooltip(|window, cx| {
                                Tooltip::new("Volume-aware activity, not measured audio amplitude.")
                                    .build(window, cx)
                            })
                            .child(div().h(rems(FADER_HEIGHT)).child(self.bars))
                            .child(
                                VolumeFader::new(&self.slider, self.label, &self.focus)
                                    .disabled(!self.enabled),
                            )
                            .child(render_fader_scale(cx)),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                h_flex()
                                    .items_baseline()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(rems(2.0))
                                            .line_height(relative(1.0))
                                            .font_bold()
                                            .child(format!("{:.0}", self.volume)),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .line_height(relative(1.0))
                                            .text_color(cx.theme().muted_foreground)
                                            .child("%"),
                                    ),
                            )
                            .when(self.volume > 100.0, |this| {
                                this.child(Tag::warning().small().child("BOOST"))
                            })
                            .when(self.muted, |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Muted"),
                                )
                            }),
                    ),
            )
            .child(self.mute.w_full())
            .child(
                h_flex().gap_2().w_full().children(
                    self.actions
                        .into_iter()
                        .map(|button| button.flex_1().min_w_0()),
                ),
            )
    }
}
