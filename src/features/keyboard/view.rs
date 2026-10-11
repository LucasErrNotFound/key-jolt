use super::KeyboardEditorView;
use super::layout::KeyboardLayout;
use gpui_kit::component::resizable::{resizable_panel, v_resizable};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::*;
use gpui_kit::*;

impl Render for KeyboardEditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let keyboard = self.render_keyboard_area(cx).into_any_element();
        let canvas_panel_height = window.rem_size()
            * match self.keyboard_layout {
                KeyboardLayout::Compact => 26.,
                KeyboardLayout::Tkl => 26.,
                KeyboardLayout::FullSize => 24.,
            };

        v_flex()
            .id("keyboard-editor")
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(cx.theme().background)
            .capture_any_mouse_down(cx.listener(
                |this, event: &MouseDownEvent, window, cx| {
                    this.begin_selection_drag(event, window, cx);
                },
            ))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                this.update_selection_drag(event, cx);
            }))
            .capture_any_mouse_up(cx.listener(|this, event: &MouseUpEvent, _, cx| {
                if event.button == MouseButton::Left {
                    this.finish_selection_drag(cx);
                }
            }))
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.finish_selection_drag(cx)),
            )
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && this.selection_drag.is_some() {
                    this.cancel_selection_drag(cx);
                    cx.stop_propagation();
                    window.prevent_default();
                }
            }))
            .child(self.render_header(cx))
            .child(
                div().flex_1().min_h_0().child(
                    v_resizable("keyboard-editor-panels")
                        .child(
                            resizable_panel()
                                .size(canvas_panel_height)
                                .size_range(canvas_panel_height..Pixels::MAX)
                                .child(
                                    div()
                                        .id("keyboard-canvas-panel")
                                        .size_full()
                                        .min_h_0()
                                        .overflow_y_scrollbar()
                                        .child(
                                            v_flex()
                                                .w_full()
                                                .min_w_0()
                                                .px_4()
                                                .py_2()
                                                .gap_2()
                                                .child(
                                                    h_flex()
                                                        .justify_between()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            TabBar::new("keyboard-layout-tabs")
                                                                .segmented()
                                                                .small()
                                                                .selected_index(
                                                                    self.keyboard_layout.index(),
                                                                )
                                                                .on_click(cx.listener(
                                                                    |this, index, _, cx| {
                                                                        this.set_layout(*index, cx);
                                                                    },
                                                                ))
                                                                .child(Tab::new().label("Full size"))
                                                                .child(Tab::new().label("TKL"))
                                                                .child(Tab::new().label("Compact")),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(
                                                                    cx.theme().muted_foreground,
                                                                )
                                                                .child(format!(
                                                                    "{}/{} mapped",
                                                                    self.mapped_key_count(),
                                                                    self.keyboard_layout
                                                                        .key_ids()
                                                                        .len(),
                                                                )),
                                                        ),
                                                )
                                                .child(keyboard)
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child("Click to select · Ctrl-click to toggle · Drag to select"),
                                                )
                                                .child(self.render_selection_controls(cx))
                                                .child(self.render_assignment_legend(cx)),
                                        ),
                                ),
                        )
                        .child(
                            resizable_panel()
                                .size_range(px(150.)..Pixels::MAX)
                                .child(self.render_inspector(cx)),
                        ),
                ),
            )
            .child(self.render_sync_footer(cx))
    }
}
