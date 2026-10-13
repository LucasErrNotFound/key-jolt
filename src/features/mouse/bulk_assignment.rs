use super::assignment_math::{
    BulkAssignmentUndo, copy_changes_assignments, copy_requires_confirmation,
};
use super::model::MouseButton;
use super::{MouseEditorEvent, MouseEditorView};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl MouseEditorView {
    fn copy_button_assignments(&mut self, source: MouseButton, cx: &mut Context<Self>) {
        let previous_count = self.assigned_sound_count();
        let mut rows: Vec<_> = self
            .files
            .iter()
            .map(|file| (file.id, file.assigned_buttons))
            .collect();
        let Some(undo) = BulkAssignmentUndo::apply(&mut rows, source.index()) else {
            return;
        };
        for (file, (_, assignments)) in self.files.iter_mut().zip(rows) {
            file.assigned_buttons = assignments;
        }
        self.bulk_undo = Some(undo);
        self.update_playback_mode_for_selection_change(previous_count);
        cx.notify();
    }

    fn undo_bulk_assignment(&mut self, cx: &mut Context<Self>) {
        let Some(undo) = self.bulk_undo.take() else {
            return;
        };
        let previous_count = self.assigned_sound_count();
        let mut rows: Vec<_> = self
            .files
            .iter()
            .map(|file| (file.id, file.assigned_buttons))
            .collect();
        if undo.restore(&mut rows) {
            cx.emit(MouseEditorEvent::StopPreviewRequested { file_id: None });
            for (file, (_, assignments)) in self.files.iter_mut().zip(rows) {
                file.assigned_buttons = assignments;
            }
            self.update_playback_mode_for_selection_change(previous_count);
        }
        cx.notify();
    }

    pub(super) fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().flex_shrink_0().w_full().px_4().py_3().gap_3()
            .border_t_1().border_color(cx.theme().border).bg(cx.theme().secondary)
            .child(self.render_playback(cx))
            .child(h_flex().gap_2().items_center()
                .child(self.render_apply_to_all(cx))
                .child(Button::new("mouse-undo-bulk-assignment")
            .when(self.bulk_undo.is_some(), |control| control.cursor_pointer()).outline().small()
                    .label("Undo apply to all")
                    .disabled(self.bulk_undo.is_none())
                    .tooltip("Restore assignments from before the last bulk action; later sound edits clear this undo")
                    .on_click(cx.listener(|this, _, _, cx| this.undo_bulk_assignment(cx)))))
    }

    pub(super) fn render_apply_to_all(&self, cx: &mut Context<Self>) -> AnyElement {
        let source = self.selected_button;
        let rows: Vec<_> = self
            .files
            .iter()
            .map(|file| file.assigned_buttons)
            .collect();
        let trigger = Button::new("mouse-apply-to-all")
            .when(copy_changes_assignments(&rows, source.index()), |control| {
                control.cursor_pointer()
            })
            .primary()
            .small()
            .label("Apply to all buttons")
            .disabled(!copy_changes_assignments(&rows, source.index()))
            .tooltip("Copy this button's sounds to the other two buttons");
        if !copy_requires_confirmation(&rows, source.index()) {
            return trigger
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.copy_button_assignments(source, cx);
                }))
                .into_any_element();
        }
        let entity = cx.entity().downgrade();
        AlertDialog::new(cx)
            .trigger(trigger)
            .on_ok(move |_, _, cx| {
                _ = entity.update(cx, |view, cx| view.copy_button_assignments(source, cx));
                true
            })
            .content(move |content, _, _| {
                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child("Replace other buttons' sounds?"))
                            .child(DialogDescription::new().child(format!(
                                "The sounds assigned to {} will replace the sound lists on the other two buttons. Audio files remain in this preset.",
                                source.label(),
                            ))),
                    )
                    .child(
                        DialogFooter::new()
                            .child(DialogClose::new().child(
                                Button::new("mouse-cancel-copy").cursor_pointer().outline().label("Keep assignments"),
                            ))
                            .child(DialogAction::new().child(
                                Button::new("mouse-confirm-copy").cursor_pointer().primary().label("Apply to all"),
                            )),
                    )
            })
            .into_any_element()
    }
}
