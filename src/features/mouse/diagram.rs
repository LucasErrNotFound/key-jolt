use super::MouseEditorView;
use super::model::MouseButton;
use gpui_kit::component::label::Label;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn render_mouse_button(
        &self,
        button: MouseButton,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected_button == button;
        let count = self
            .files
            .iter()
            .filter(|file| file.assigned_buttons[button.index()])
            .count();
        let entity = cx.entity().downgrade();

        h_flex()
            .id(format!("mouse-button-row-{}", button.index()))
            .w_full()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .py_2()
            .border_1()
            .rounded(cx.theme().radius)
            .border_color(if is_selected {
                cx.theme().primary
            } else {
                cx.theme().muted_foreground.opacity(0.25)
            })
            .when(is_selected, |this| {
                this.bg(cx.theme().primary.opacity(0.07))
            })
            .child(Label::new(label.into()).font_medium())
            .child(
                Label::new(if count == 0 {
                    "Not set".to_string()
                } else {
                    format!("{count} sound{}", if count == 1 { "" } else { "s" })
                })
                .text_sm()
                .text_color(cx.theme().muted_foreground),
            )
            .on_click(move |_, _, cx| {
                _ = entity.update(cx, |view, cx| view.select_button(button, cx));
            })
    }

    pub(super) fn render_mouse_diagram(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let left_active = self.selected_button == MouseButton::Left;
        let right_active = self.selected_button == MouseButton::Right;
        let middle_active = self.selected_button == MouseButton::Middle;
        let entity = cx.entity().downgrade();
        let left_entity = entity.clone();
        let right_entity = entity.clone();
        let middle_entity = entity;
        let highlight_color = cx.theme().primary.opacity(0.14);
        let diagram_unit = cx.theme().font_size;

        v_flex()
            .relative()
            .w(rems(11.25))
            .h(rems(16.875))
            .rounded_full()
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.35))
            .overflow_hidden()
            .child(
                canvas(
                    |bounds, _, _| bounds,
                    move |_, bounds, window, _| {
                        let origin = bounds.origin;
                        let top = origin.y + px(1.0);
                        let left = origin.x + px(1.0);
                        let right = origin.x + bounds.size.width - px(1.0);
                        let center = origin.x + bounds.size.width / 2.;
                        let radius = bounds.size.width / 2. - px(1.0);
                        let control_factor = 0.552_284_8;
                        let divider_left = center - diagram_unit * 1.8125 / 2.;
                        let divider_right = center + diagram_unit * 1.8125 / 2.;
                        let button_bottom = top + diagram_unit * 7.1875;

                        if left_active {
                            let mut path = PathBuilder::fill();
                            path.move_to(point(center, top));
                            path.cubic_bezier_to(
                                point(left, top + radius),
                                point(center - radius * control_factor, top),
                                point(left, top + radius * (1. - control_factor)),
                            );
                            path.line_to(point(left, button_bottom));
                            path.line_to(point(divider_left, button_bottom));
                            path.line_to(point(divider_left, top));
                            path.close();
                            if let Ok(path) = path.build() {
                                window.paint_path(path, highlight_color);
                            }
                        }

                        if right_active {
                            let mut path = PathBuilder::fill();
                            path.move_to(point(center, top));
                            path.cubic_bezier_to(
                                point(right, top + radius),
                                point(center + radius * control_factor, top),
                                point(right, top + radius * (1. - control_factor)),
                            );
                            path.line_to(point(right, button_bottom));
                            path.line_to(point(divider_right, button_bottom));
                            path.line_to(point(divider_right, top));
                            path.close();
                            if let Ok(path) = path.build() {
                                window.paint_path(path, highlight_color);
                            }
                        }
                    },
                )
                .absolute()
                .inset_0()
                .size_full(),
            )
            .child(
                h_flex()
                    .mx(px(1.0))
                    .mt(px(1.0))
                    .h(rems(7.1875))
                    .items_stretch()
                    .child(
                        div()
                            .id("mouse-left-zone")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Label::new("LEFT")
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .on_click(move |_, _, cx| {
                                _ = left_entity.update(cx, |view, cx| {
                                    view.select_button(MouseButton::Left, cx)
                                });
                            }),
                    )
                    .child(
                        v_flex()
                            .id("mouse-middle-zone")
                            .w(rems(1.8125))
                            .h_full()
                            .items_center()
                            .justify_center()
                            .border_x_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.25))
                            .bg(cx.theme().muted.opacity(0.12))
                            .child(
                                div()
                                    .id("mouse-middle-wheel")
                                    .w(rems(0.875))
                                    .h(rems(2.25))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(if middle_active {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().muted_foreground.opacity(0.45)
                                    })
                                    .bg(if middle_active {
                                        cx.theme().primary.opacity(0.22)
                                    } else {
                                        cx.theme().background
                                    }),
                            )
                            .on_click(move |_, _, cx| {
                                _ = middle_entity.update(cx, |view, cx| {
                                    view.select_button(MouseButton::Middle, cx)
                                });
                            }),
                    )
                    .child(
                        div()
                            .id("mouse-right-zone")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Label::new("RIGHT")
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .on_click(move |_, _, cx| {
                                _ = right_entity.update(cx, |view, cx| {
                                    view.select_button(MouseButton::Right, cx)
                                });
                            }),
                    ),
            )
            .child(
                div()
                    .mx(px(1.0))
                    .mb(px(1.0))
                    .flex_1()
                    .border_t_1()
                    .border_color(cx.theme().muted_foreground.opacity(0.25))
                    .bg(cx.theme().muted.opacity(0.12)),
            )
    }
}
