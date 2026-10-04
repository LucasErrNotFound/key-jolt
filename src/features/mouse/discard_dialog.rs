use super::{MouseEditorEvent, MouseEditorView};

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};

use gpui_kit::component::*;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn render_back_control(&self, cx: &mut Context<Self>) -> AnyElement {
        let back_entity = cx.entity().downgrade();
        if self.has_unsaved_changes(cx) {
            let confirm_back = back_entity.clone();
            AlertDialog::new(cx)
                .trigger(
                    Button::new("mouse-editor-back")
                        .outline()
                        .icon(IconName::ArrowLeft)
                        .accessibility_label("Back to home")
                        .tooltip("Back"),
                )
                .on_ok(move |_, _, cx| {
                    _ = confirm_back.update(cx, |_view, cx| {
                        cx.emit(MouseEditorEvent::BackRequested);
                    });
                    true
                })
                .content(|content, _, cx| {
                    content
                        .child(
                            DialogHeader::new()
                                .items_center()
                                .child(
                                    Icon::new(IconName::TriangleAlert)
                                        .with_size(cx.theme().font_size * 1.5)
                                        .text_color(cx.theme().warning),
                                )
                                .child(
                                    v_flex()
                                        .w_full()
                                        .items_center()
                                        .text_center()
                                        .gap_1()
                                        .child(DialogTitle::new().child("Discard unsaved changes?"))
                                        .child(
                                            DialogDescription::new()
                                                .child("Your mouse preset changes will be lost."),
                                        ),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    DialogClose::new().child(
                                        Button::new("mouse-keep-editing")
                                            .outline()
                                            .icon(IconName::Pencil)
                                            .label("Keep editing"),
                                    ),
                                )
                                .child(
                                    DialogAction::new().child(
                                        Button::new("mouse-discard-changes")
                                            .danger()
                                            .icon(IconName::ArrowLeft)
                                            .label("Discard changes"),
                                    ),
                                ),
                        )
                })
                .into_any_element()
        } else {
            Button::new("mouse-editor-back")
                .outline()
                .icon(IconName::ArrowLeft)
                .accessibility_label("Back to home")
                .tooltip("Back")
                .on_click(move |_, _, cx| {
                    _ = back_entity
                        .update(cx, |_view, cx| cx.emit(MouseEditorEvent::BackRequested));
                })
                .into_any_element()
        }
    }
}
