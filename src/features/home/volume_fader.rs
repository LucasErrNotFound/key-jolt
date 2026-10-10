use gpui_kit::base::{Slider, SliderIndicator, SliderThumb, SliderTrack};
use gpui_kit::component::slider::SliderState;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(super) const FADER_HEIGHT: f32 = 20.0;

pub(super) fn new_volume_slider(cx: &mut App, volume: f32) -> Entity<SliderState> {
    cx.new(|_| {
        SliderState::new()
            .min(0.0)
            .max(200.0)
            .step(1.0)
            .default_value(volume)
    })
}

#[derive(IntoElement)]
pub(super) struct VolumeFader {
    state: Entity<SliderState>,
    label: &'static str,
    focus: FocusHandle,
    disabled: bool,
}

impl VolumeFader {
    pub(super) fn new(
        state: &Entity<SliderState>,
        label: &'static str,
        focus: &FocusHandle,
    ) -> Self {
        Self {
            state: state.clone(),
            label,
            focus: focus.clone(),
            disabled: false,
        }
    }

    pub(super) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for VolumeFader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let position = self.state.read(cx).percentage().end;
        let key_state = self.state.clone();
        let focus = self.focus.clone();
        let disabled = self.disabled;
        let track = SliderTrack::new(&self.state)
            .disabled(disabled)
            .axis(Axis::Vertical)
            .relative()
            .size_full()
            .child(
                SliderIndicator::new(&self.state)
                    .absolute()
                    .inset_0()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(rems(0.75))
                            .w(rems(0.5))
                            .rounded_full()
                            .bg(cx.theme().yellow.opacity(0.25)),
                    )
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left(rems(0.75))
                            .w(rems(0.5))
                            .h(relative(position))
                            .rounded_full()
                            .bg(cx.theme().primary),
                    )
                    .child(
                        div()
                            .absolute()
                            .bottom(relative(0.5))
                            .left_0()
                            .w_full()
                            .h(px(1.0))
                            .bg(cx.theme().muted_foreground),
                    ),
            )
            .child(
                SliderThumb::new(&self.state)
                    .disabled(disabled)
                    .axis(Axis::Vertical)
                    .absolute()
                    .bottom(relative(position))
                    .left_0()
                    .mb(rems(-0.375))
                    .w_full()
                    .h(rems(0.75))
                    .rounded(rems(0.125))
                    .bg(cx.theme().foreground)
                    .border_1()
                    .border_color(cx.theme().border)
                    .on_mouse_down(MouseButton::Left, {
                        let focus = self.focus.clone();
                        move |_, window, cx| {
                            if !disabled {
                                focus.focus(window, cx);
                            }
                        }
                    }),
            );
        let control = if disabled {
            div().size_full().child(track).into_any_element()
        } else {
            Slider::new(&self.state)
                .vertical()
                .size_full()
                .child(track)
                .into_any_element()
        };
        div()
            .id("volume-fader-track")
            .track_focus(&self.focus)
            .tab_stop(!disabled)
            .aria_label(format!("{} volume", self.label))
            .focus(|style| style.border_1().border_color(cx.theme().ring))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if !disabled {
                    focus.focus(window, cx);
                }
            })
            .on_key_down(
                move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                    if disabled {
                        return;
                    }
                    let current = key_state.read(&*cx).value().start();
                    let step = if event.keystroke.modifiers.shift {
                        10.0
                    } else {
                        1.0
                    };
                    let value = match event.keystroke.key.as_str() {
                        "up" | "right" => current + step,
                        "down" | "left" => current - step,
                        "home" => 0.0,
                        "end" => 200.0,
                        _ => return,
                    };
                    key_state.update(cx, |state, cx| {
                        state.set_value(value.clamp(0.0, 200.0), window, cx);
                        state.handle_release(cx);
                    });
                    cx.stop_propagation();
                },
            )
            .h(rems(FADER_HEIGHT))
            .w(rems(2.0))
            .child(control)
    }
}

pub(super) fn render_fader_scale(cx: &App) -> impl IntoElement {
    div()
        .relative()
        .h(rems(FADER_HEIGHT))
        .w(rems(2.25))
        .flex_shrink_0()
        .text_xs()
        .text_center()
        .text_color(cx.theme().muted_foreground)
        .children([200, 175, 150, 125, 100, 75, 50, 25, 0].map(|value| {
            div()
                .absolute()
                .left_0()
                .top(relative((200 - value) as f32 / 200.0))
                .mt(rems(-0.5))
                .w_full()
                .h(rems(1.0))
                .flex()
                .items_center()
                .justify_center()
                .text_center()
                .line_height(relative(1.0))
                .when(value == 100, |this| {
                    this.text_color(cx.theme().foreground).font_semibold()
                })
                .child(value.to_string())
        }))
}
