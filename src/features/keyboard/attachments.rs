use super::assignment::{SelectionAssignment, assignment_state};
use super::model::AudioFile;
use super::{KeyboardEditorEvent, KeyboardEditorView};
use crate::features::preset_editor::{PreviewState, UploadState};
use gpui_kit::assets::IconName;
use gpui_kit::base::{Checkbox, CheckboxIndicator, CheckboxState};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn render_file_row(
        &self,
        file: AudioFile,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = file.id;
        let selection = &self.selected_keys[self.keyboard_layout.index()];
        let state =
            match assignment_state(&file.assigned_keys[self.keyboard_layout.index()], selection) {
                SelectionAssignment::None => CheckboxState::Unchecked,
                SelectionAssignment::All => CheckboxState::Checked,
                SelectionAssignment::Mixed => CheckboxState::Indeterminate,
            };
        let ready = matches!(file.state, UploadState::Success | UploadState::Complete);
        let playing = file.preview_state == PreviewState::Playing;
        let disabled = selection.is_empty() || !ready;
        let checkbox_entity = cx.entity().downgrade();
        let preview_entity = cx.entity().downgrade();
        let path = file.path.clone();
        let checkbox = Checkbox::new(format!("assign-sound-{id}"))
            .state(state)
            .disabled(disabled)
            .accessibility_label(format!("Assign {} to selected keys", file.name))
            .on_change(move |state, _, _, cx| {
                _ = checkbox_entity.update(cx, |view, cx| view.set_file_assignment(id, state, cx));
            })
            .child(
                CheckboxIndicator::new()
                    .state(state)
                    .disabled(disabled)
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_4()
                    .border_1()
                    .border_color(cx.theme().input)
                    .rounded(cx.theme().radius_tokens().sm)
                    .when(state != CheckboxState::Unchecked, |indicator| {
                        indicator
                            .bg(cx.theme().primary)
                            .border_color(cx.theme().primary)
                            .child(if state == CheckboxState::Checked {
                                Icon::new(IconName::Check)
                                    .with_size(px(12.))
                                    .text_color(cx.theme().primary_foreground)
                                    .into_any_element()
                            } else {
                                div()
                                    .text_color(cx.theme().primary_foreground)
                                    .child("−")
                                    .into_any_element()
                            })
                    }),
            );
        let preview = Button::new(format!("preview-file-{id}"))
            .secondary()
            .small()
            .icon(if playing {
                IconName::Close
            } else {
                IconName::Play
            })
            .disabled(!ready)
            .accessibility_label(format!(
                "{} preview of {}",
                if playing { "Stop" } else { "Play" },
                file.name
            ))
            .tooltip(if playing {
                "Stop preview"
            } else {
                "Play preview"
            })
            .on_click(move |_, _, cx| {
                _ = preview_entity.update(cx, |_, cx| {
                    if playing {
                        cx.emit(KeyboardEditorEvent::StopPreviewRequested { file_id: Some(id) });
                    } else {
                        cx.emit(KeyboardEditorEvent::PreviewRequested {
                            file_id: id,
                            path: path.clone(),
                        });
                    }
                });
            });
        v_flex()
            .w_full()
            .p_3()
            .gap_2()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius_tokens().lg)
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(preview)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .truncate()
                                    .child(file.name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(match &file.preview_state {
                                        PreviewState::Error(message) => message.clone(),
                                        _ => match file.state {
                                            UploadState::Uploading => "Uploading…".into(),
                                            UploadState::Failed => "Upload failed".into(),
                                            _ => file
                                                .size
                                                .clone()
                                                .unwrap_or_else(|| "Audio file".into()),
                                        },
                                    }),
                            ),
                    )
                    .child(div().flex_shrink_0().child(checkbox))
                    .child(self.render_delete_sound(&file, cx)),
            )
            .child(if ready {
                self.render_waveform(id, playing, cx)
            } else if file.state == UploadState::Uploading {
                Spinner::new().small().into_any_element()
            } else {
                div()
                    .text_xs()
                    .text_color(cx.theme().danger)
                    .child("Try uploading this file again.")
                    .into_any_element()
            })
            .when(
                !selection.is_empty() && state != CheckboxState::Unchecked,
                |row| {
                    row.child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if state == CheckboxState::Indeterminate {
                                        "Assigned to some selected keys"
                                    } else {
                                        "Assigned to all selected keys"
                                    }),
                            )
                            .child(
                                Button::new(format!("unassign-sound-{id}"))
                                    .ghost()
                                    .xsmall()
                                    .label("Unassign")
                                    .tooltip("Unassign from selected keys only")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_file_assignment(id, CheckboxState::Unchecked, cx);
                                    })),
                            ),
                    )
                },
            )
    }

    fn render_delete_sound(&self, file: &AudioFile, cx: &mut Context<Self>) -> AnyElement {
        let id = file.id;
        let button = Button::new(format!("delete-sound-{id}"))
            .danger()
            .outline()
            .small()
            .icon(IconName::Trash)
            .accessibility_label(format!("Delete {} from preset", file.name))
            .tooltip("Delete sound from preset");
        if file.assigned_keys.iter().any(|keys| !keys.is_empty())
            || file
                .independent_assigned_keys
                .as_ref()
                .is_some_and(|layouts| layouts.iter().any(|keys| !keys.is_empty()))
        {
            let entity = cx.entity().downgrade();
            let name = file.name.clone();
            AlertDialog::new(cx).trigger(button)
                .on_ok(move |_, _, cx| {
                    _ = entity.update(cx, |view, cx| view.remove_file(id, cx));
                    true
                })
                .content(move |content, _, _| {
                    content.child(DialogHeader::new()
                        .child(DialogTitle::new().child("Delete sound from preset?"))
                        .child(DialogDescription::new().child(format!("{} will be removed from every keyboard size. Use Unassign to remove it only from selected keys.", name))))
                        .child(DialogFooter::new()
                            .child(DialogClose::new().child(Button::new(format!("cancel-delete-sound-{id}")).outline().label("Keep sound")))
                            .child(DialogAction::new().child(Button::new(format!("confirm-delete-sound-{id}")).danger().label("Delete sound"))))
                }).into_any_element()
        } else {
            button
                .on_click(cx.listener(move |this, _, _, cx| this.remove_file(id, cx)))
                .into_any_element()
        }
    }

    pub(super) fn render_upload_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("keyboard-audio-drop-zone")
            .outline()
            .w_full()
            .h(rems(2.5))
            .cursor_pointer()
            .border_dashed()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius_tokens().lg)
            .bg(cx.theme().background)
            .text_color(cx.theme().muted_foreground)
            .icon(IconName::Upload)
            .label("Add or drop audio files")
            .accessibility_label("Add or drop audio files")
            .tooltip("Choose audio files or drop them here, then assign them to selected keys")
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                this.begin_uploads(paths.paths().to_vec(), window, cx);
            }))
            .on_click(cx.listener(|this, _, window, cx| {
                this.open_file_picker(window, cx);
            }))
    }
}
