use super::MouseEditorView;
use super::model::MouseButton;
use gpui_kit::component::button::Button;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn render_inspector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let button = self.selected_button;
        let assigned_rows: Vec<_> = self
            .files
            .iter()
            .filter(|file| file.assigned_buttons[button.index()])
            .cloned()
            .map(|file| self.render_file_row(file, false, cx).into_any_element())
            .collect();
        let catalog_rows: Vec<_> = if self.show_sound_catalog {
            self.files
                .iter()
                .cloned()
                .map(|file| self.render_file_row(file, true, cx).into_any_element())
                .collect()
        } else {
            Vec::new()
        };
        v_flex().flex_1().min_h_0().w_full().border_t_1().border_color(cx.theme().border)
            .rounded_tl(cx.theme().radius_tokens().lg).rounded_tr(cx.theme().radius_tokens().lg)
            .bg(cx.theme().secondary)
            .child(div().id("mouse-inspector-scroll").flex_1().min_h_0().overflow_y_scrollbar()
                .child(v_flex().w_full().min_w_0().p_4().gap_3()
                    .child(h_flex().w_full().gap_2().justify_between().items_center()
                        .child(v_flex().flex_1().min_w_0().gap_1()
                            .child(div().text_lg().font_semibold().child(button.label()))
                            .child(div().text_xs().text_color(cx.theme().muted_foreground)
                                .child(match button {
                                    MouseButton::Left => "Plays when you press the left button.",
                                    MouseButton::Right => "Plays when you press the right button.",
                                    MouseButton::Middle => "Plays when you press the wheel, not when scrolling.",
                                })))
                    )
                    .when(assigned_rows.is_empty(), |panel| panel.child(
                        div().p_3().rounded(cx.theme().radius_tokens().lg).border_1().border_color(cx.theme().border)
                            .text_sm().text_color(cx.theme().muted_foreground)
                            .child("No sounds assigned. Add audio or choose a sound from the catalog below.")))
                    .children(assigned_rows)
                    .child(self.render_upload_area(cx))
                    .when(!self.files.is_empty(), |panel| panel.child(
                        Button::new("mouse-toggle-sound-catalog").cursor_pointer().outline().small().w_full()
                            .tooltip(if self.show_sound_catalog {
                                "Hide the catalog and show this button's assigned sounds"
                            } else {
                                "Show all preset sounds to assign to this button"
                            })
                            .label(if self.show_sound_catalog { "Hide sound catalog" } else { "Show sound catalog" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_sound_catalog = !this.show_sound_catalog;
                                cx.notify();
                            }))))
                    .when(self.show_sound_catalog && !self.files.is_empty(), |panel| {
                        panel.child(v_flex().gap_2()
                            .child(div().text_sm().font_semibold().child("Sound catalog"))
                            .child(div().text_xs().text_color(cx.theme().muted_foreground)
                                .child("Check sounds to assign them to this button. Trash deletes from the entire preset."))
                            .children(catalog_rows))
                    })
                ))
    }
}
