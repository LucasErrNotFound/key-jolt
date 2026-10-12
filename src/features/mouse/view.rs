use super::MouseEditorView;
use gpui_kit::component::*;
use gpui_kit::*;

impl Render for MouseEditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .min_h_0()
            .bg(cx.theme().background)
            .child(self.render_header(cx))
            .child(self.render_mouse_canvas(cx))
            .child(self.render_inspector(cx))
            .child(self.render_footer(cx))
    }
}
