use super::{KeyboardEditorEvent, KeyboardEditorView};

use super::assignment::{SelectionAssignment, assignment_state};
use super::model::AudioFile;
use crate::features::preset_editor::{PreviewState, UploadState};

use gpui_kit::assets::IconName;
use gpui_kit::base::{Checkbox, CheckboxIndicator, CheckboxState};

use gpui_kit::component::attachment::{
    Attachment, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentMedia,
    AttachmentStatus, AttachmentTitle,
};

use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
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
        let checked = state == CheckboxState::Checked;
        let mixed = state == CheckboxState::Indeterminate;
        let disabled = selection.is_empty()
            || !matches!(file.state, UploadState::Success | UploadState::Complete);
        let hovered = self.hovered_file == Some(id);
        let entity = cx.entity().downgrade();

        let media_content = match file.state {
            UploadState::Uploading => Spinner::new()
                .small()
                .color(cx.theme().primary)
                .into_any_element(),

            UploadState::Success => Icon::new(IconName::CircleCheck)
                .with_size(cx.theme().font_size * 1.125)
                .text_color(cx.theme().green)
                .into_any_element(),

            UploadState::Complete => {
                let preview_entity = entity.clone();
                let path = file.path.clone();
                let is_playing = file.preview_state == PreviewState::Playing;
                let preview_button = Button::new(format!("preview-file-{id}"))
                    .ghost()
                    .xsmall()
                    .icon(if is_playing {
                        IconName::Close
                    } else if hovered {
                        IconName::Play
                    } else {
                        IconName::FileVolume
                    })
                    .accessibility_label(if is_playing {
                        format!("Stop preview of {}", file.name)
                    } else {
                        format!("Play preview of {}", file.name)
                    })
                    .tooltip(if is_playing {
                        "Stop preview"
                    } else {
                        "Play preview"
                    })
                    .on_click(move |_event, _window, cx| {
                        _ = preview_entity.update(cx, |_, cx| {
                            if is_playing {
                                cx.emit(KeyboardEditorEvent::StopPreviewRequested {
                                    file_id: Some(id),
                                });
                            } else {
                                cx.emit(KeyboardEditorEvent::PreviewRequested {
                                    file_id: id,
                                    path: path.clone(),
                                });
                            }
                        });
                    });

                div()
                    .id(format!("preview-hover-{id}"))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_hover({
                        let entity = entity.clone();

                        move |is_hovered, _window, cx| {
                            _ = entity.update(cx, |view, cx| {
                                if *is_hovered {
                                    view.hovered_file = Some(id);
                                } else if view.hovered_file == Some(id) {
                                    view.hovered_file = None;
                                }
                                cx.notify();
                            });
                        }
                    })
                    .child(preview_button)
                    .into_any_element()
            }

            UploadState::Failed => Icon::new(IconName::CircleX)
                .with_size(cx.theme().font_size * 1.125)
                .text_color(cx.theme().red)
                .into_any_element(),
        };

        let media = AttachmentMedia::new().child(media_content);

        let attachment_status = match file.state {
            UploadState::Uploading => AttachmentStatus::Uploading,
            UploadState::Success | UploadState::Complete => AttachmentStatus::Complete,
            UploadState::Failed => AttachmentStatus::Failed,
        };

        let description = match file.state {
            UploadState::Uploading => SharedString::from("Uploading"),
            UploadState::Success | UploadState::Complete => match &file.preview_state {
                PreviewState::Error(message) => message.clone(),
                _ => file.size.clone().unwrap_or_else(|| "Preparing...".into()),
            },
            UploadState::Failed => SharedString::from("Upload failed"),
        };

        let remove_entity = cx.entity().downgrade();
        let checkbox_entity = cx.entity().downgrade();

        let attachment = Attachment::new()
            .small()
            .status(attachment_status)
            .flex_1()
            .media(AttachmentMedia::new().child(media))
            .content(
                AttachmentContent::new()
                    .title(AttachmentTitle::new(file.name.clone()))
                    .description(AttachmentDescription::new(description)),
            )
            .actions(
                AttachmentActions::new().child(
                    Button::new(format!("remove-file-{id}"))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Close)
                        .accessibility_label(format!("Remove {}", file.name))
                        .tooltip("Remove")
                        .on_click(move |_event, _window, cx| {
                            _ = remove_entity.update(cx, |view, cx| {
                                view.remove_file(id, cx);
                            });
                        }),
                ),
            );

        let checkbox = Checkbox::new(format!("select-file-{id}"))
            .state(state)
            .disabled(disabled)
            .accessibility_label(format!("Assign {} to selected keys", file.name))
            .on_change(move |state, _window, _cx, cx| {
                _ = checkbox_entity.update(cx, |view, cx| {
                    view.set_file_assignment(id, state, cx);
                });
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
                    .border_color(cx.theme().muted_foreground.opacity(0.4))
                    .when(checked, |this| {
                        this.bg(cx.theme().primary)
                            .border_color(cx.theme().primary)
                            .child(
                                Icon::new(IconName::Check)
                                    .with_size(cx.theme().font_size * 0.75)
                                    .text_color(cx.theme().primary_foreground),
                            )
                    })
                    .when(mixed, |this| {
                        this.bg(cx.theme().primary)
                            .border_color(cx.theme().primary)
                            .text_color(cx.theme().primary_foreground)
                            .child("−")
                    }),
            );

        h_flex()
            .items_center()
            .gap_3()
            .w_full()
            .child(attachment)
            .child(
                div()
                    .w(rems(1.75))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(checkbox),
            )
    }

    pub(super) fn render_upload_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Empty::new()
            .w_full()
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.25))
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Icon)
                            .child(
                                Icon::new(IconName::Upload).with_size(cx.theme().font_size * 1.375),
                            ),
                    )
                    .title(EmptyTitle::new().child("Add audio files"))
                    .description(EmptyDescription::new().child("Select one or more audio files.")),
            )
            .content(
                EmptyContent::new().flex_row().justify_center().child(
                    Button::new("upload-files")
                        .primary()
                        .label("Upload Files...")
                        .on_click(cx.listener(|this, _event, window, cx| {
                            this.open_file_picker(window, cx);
                        })),
                ),
            )
    }
}
