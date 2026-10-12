use super::MouseEditorView;
use super::diagram_geometry::click_geometry;
use super::model::MouseButton;
use gpui_kit::base::Button as HitButton;
use gpui_kit::component::button::Button;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn button_color(button: MouseButton, cx: &App) -> Hsla {
        match button {
            MouseButton::Left => cx.theme().blue,
            MouseButton::Right => cx.theme().green,
            MouseButton::Middle => cx.theme().magenta,
        }
    }

    fn button_sound_count(&self, button: MouseButton) -> usize {
        self.files
            .iter()
            .filter(|file| file.assigned_buttons[button.index()])
            .count()
    }

    fn render_button_card(&self, button: MouseButton, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.button_sound_count(button);
        let selected = self.selected_button == button;
        let color = Self::button_color(button, cx);
        let summary = if count == 0 {
            "No sounds assigned".to_string()
        } else {
            format!("{count} sound{}", if count == 1 { "" } else { "s" })
        };
        Button::new(format!("mouse-button-card-{}", button.index()))
            .outline()
            .w_full()
            .min_w_0()
            .h(rems(3.5))
            .rounded(cx.theme().radius_tokens().lg)
            .bg(if count > 0 {
                color.opacity(0.08)
            } else {
                cx.theme().muted.opacity(0.25)
            })
            .border_color(if selected {
                cx.theme().primary
            } else {
                cx.theme().border
            })
            .when(selected, |card| card.border_2())
            .accessibility_label(format!(
                "{}, {}{}",
                button.label(),
                summary,
                if selected { ", selected" } else { "" },
            ))
            .tooltip(format!("Edit sounds for {}", button.label()))
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .items_start()
                    .child(
                        div()
                            .mt(rems(0.3))
                            .size(rems(0.5))
                            .flex_shrink_0()
                            .rounded_full()
                            .bg(color),
                    )
                    .child(
                        v_flex()
                            .min_w_0()
                            .flex_1()
                            .items_start()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .truncate()
                                    .child(button.label()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .truncate()
                                    .child(summary),
                            ),
                    ),
            )
            .on_click(cx.listener(move |this, _, _, cx| this.select_button(button, cx)))
    }

    fn render_click_zone(&self, button: MouseButton, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.selected_button == button;
        let primary = cx.theme().primary;
        let label = SharedString::from(button.label());
        HitButton::new(format!("mouse-zone-{}", button.index()))
            .flex_1()
            .min_w_0()
            .h_full()
            .text_sm()
            .selected(selected)
            .text_color(cx.theme().muted_foreground)
            .styles(|styles| styles.selected(|style| style.text_color(primary)))
            .hover(|style| style.text_color(primary))
            .focus(|style| style.text_color(primary).font_weight(FontWeight::BOLD))
            .accessibility_label(format!("Select {}", button.label()))
            .tooltip(move |window, cx| Tooltip::new(label.clone()).build(window, cx))
            .child(if button == MouseButton::Left {
                "L"
            } else {
                "R"
            })
            .on_click(cx.listener(move |this, _, _, cx| this.select_button(button, cx)))
    }

    fn render_click_fills(&self, cx: &App) -> impl IntoElement {
        let fill = |button| {
            let color = Self::button_color(button, cx);
            if self.selected_button == button {
                color.opacity(0.3)
            } else if self.button_sound_count(button) > 0 {
                color.opacity(0.18)
            } else {
                cx.theme().muted.opacity(0.25)
            }
        };
        let left_fill = fill(MouseButton::Left);
        let right_fill = fill(MouseButton::Right);
        let border = cx.theme().border;
        canvas(
            |bounds, window, _| {
                click_geometry(
                    f32::from(bounds.size.width),
                    f32::from(window.rem_size()) * 6.,
                    f32::from(window.rem_size()) * 1.75,
                )
            },
            move |bounds, geometry, window, _| {
                let Some(g) = geometry else {
                    return;
                };
                let at = |x, y| point(bounds.origin.x + px(x), bounds.origin.y + px(y));
                let radius = point(px(g.radius), px(g.radius));
                let mut left = PathBuilder::fill();
                left.move_to(at(g.left, g.bottom));
                left.line_to(at(g.left, g.curve_y));
                left.arc_to(radius, px(0.), false, true, at(g.inner_left, g.cut_y));
                left.line_to(at(g.inner_left, g.bottom));
                left.close();
                if let Ok(path) = left.build() {
                    window.paint_path(path, left_fill);
                }
                let mut right = PathBuilder::fill();
                right.move_to(at(g.inner_right, g.bottom));
                right.line_to(at(g.inner_right, g.cut_y));
                right.arc_to(radius, px(0.), false, true, at(g.right, g.curve_y));
                right.line_to(at(g.right, g.bottom));
                right.close();
                if let Ok(path) = right.build() {
                    window.paint_path(path, right_fill);
                }
                for x in [g.inner_left, g.inner_right] {
                    let mut divider = PathBuilder::stroke(px(1.));
                    divider.move_to(at(x, g.cut_y));
                    divider.line_to(at(x, g.bottom));
                    if let Ok(path) = divider.build() {
                        window.paint_path(path, border);
                    }
                }
            },
        )
        .absolute()
        .inset_0()
        .size_full()
    }

    fn render_mouse_diagram(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let color = Self::button_color(MouseButton::Middle, cx);
        let middle_selected = self.selected_button == MouseButton::Middle;
        let middle_assigned = self.button_sound_count(MouseButton::Middle) > 0;
        let offset = if cx.reduce_motion() {
            0.
        } else {
            self.wheel_offset
        };
        v_flex()
            .relative()
            .w(rems(8.5))
            .h(rems(12.5))
            .flex_shrink_0()
            .rounded_full()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(self.render_click_fills(cx))
            .child(
                h_flex()
                    .w_full()
                    .h(rems(6.))
                    .flex_shrink_0()
                    .items_stretch()
                    .child(self.render_click_zone(MouseButton::Left, cx))
                    .child(
                        HitButton::new("mouse-middle-zone")
                            .w(rems(1.75))
                            .h_full()
                            .accessibility_label("Select middle click")
                            .focus(|style| style.opacity(0.65))
                            .tooltip(|window, cx| {
                                Tooltip::new("Middle click · scroll here to turn the illustration")
                                    .build(window, cx)
                            })
                            .child(
                                div()
                                    .relative()
                                    .w(rems(0.8))
                                    .h(rems(2.3))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(if middle_selected {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().border
                                    })
                                    .bg(if middle_assigned || middle_selected {
                                        color.opacity(0.3)
                                    } else {
                                        cx.theme().secondary
                                    })
                                    .child(
                                        canvas(
                                            |bounds, window, _| {
                                                (bounds, f32::from(window.rem_size()))
                                            },
                                            move |_, (bounds, rem), window, _| {
                                                let width = f32::from(bounds.size.width);
                                                let height = f32::from(bounds.size.height);
                                                let radius = width / 2. - 1.;
                                                if radius <= 0. {
                                                    return;
                                                }
                                                for index in 0..9 {
                                                    let y =
                                                        (index as f32 * 0.3 + offset - 0.3) * rem;
                                                    if y < 1. || y > height - 1. {
                                                        continue;
                                                    }
                                                    let center_y = if y < radius + 1. {
                                                        radius + 1.
                                                    } else if y > height - radius - 1. {
                                                        height - radius - 1.
                                                    } else {
                                                        y
                                                    };
                                                    let half_width = (radius * radius
                                                        - (y - center_y).powi(2))
                                                    .max(0.)
                                                    .sqrt();
                                                    let mut ridge = PathBuilder::stroke(px(1.));
                                                    ridge.move_to(point(
                                                        bounds.origin.x
                                                            + px(width / 2. - half_width),
                                                        bounds.origin.y + px(y),
                                                    ));
                                                    ridge.line_to(point(
                                                        bounds.origin.x
                                                            + px(width / 2. + half_width),
                                                        bounds.origin.y + px(y),
                                                    ));
                                                    if let Ok(path) = ridge.build() {
                                                        window
                                                            .paint_path(path, color.opacity(0.55));
                                                    }
                                                }
                                            },
                                        )
                                        .absolute()
                                        .inset_0()
                                        .size_full(),
                                    ),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_button(MouseButton::Middle, cx)
                            }))
                            .on_scroll_wheel(cx.listener(
                                |this, event: &ScrollWheelEvent, window, cx| {
                                    this.turn_wheel(event, window, cx)
                                },
                            )),
                    )
                    .child(self.render_click_zone(MouseButton::Right, cx)),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .border_t_1()
                    .border_color(cx.theme().border),
            )
    }

    pub(super) fn render_mouse_canvas(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().flex_shrink_0().px_4().py_3().gap_2().child(
            v_flex()
                .w_full()
                .p_3()
                .gap_3()
                .rounded(cx.theme().radius_tokens().lg)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().muted.opacity(0.1))
                .child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .items_center()
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(self.render_button_card(MouseButton::Left, cx)),
                        )
                        .child(div().w(rems(1.)).h(px(1.)).bg(cx.theme().border))
                        .child(self.render_mouse_diagram(cx))
                        .child(
                            v_flex()
                                .w(rems(1.))
                                .h(rems(8.25))
                                .py(rems(1.75))
                                .justify_between()
                                .child(div().w_full().h(px(1.)).bg(cx.theme().border))
                                .child(div().w_full().h(px(1.)).bg(cx.theme().border)),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_5()
                                .child(self.render_button_card(MouseButton::Middle, cx))
                                .child(self.render_button_card(MouseButton::Right, cx)),
                        ),
                )
                .child(
                    div()
                        .text_center()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(
                            "Select a button to edit its sounds. Scroll over the wheel to turn it.",
                        ),
                ),
        )
    }
}
