use super::MouseEditorEvent;
use super::MouseEditorView;
use super::model::AudioFile;
use crate::features::preset_editor::{PreviewState, UploadState, render_audio_waveform};
use gpui_kit::assets::IconName;
use gpui_kit::base::{Checkbox, CheckboxIndicator, CheckboxState};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn render_file_row(
        &self,
        file: AudioFile,
        catalog: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = file.id;
        let button = self.selected_button;
        let ready = matches!(file.state, UploadState::Success | UploadState::Complete);
        let playing = file.preview_state == PreviewState::Playing;
        let assigned = file.assigned_buttons[button.index()];
        let color = Self::button_color(button, cx);
        let path = file.path.clone();
        let preview = Button::new(format!("mouse-preview-{}-{id}", u8::from(catalog)))
            .secondary()
            .small()
            .w(rems(1.75))
            .h(rems(1.75))
            .rounded(cx.theme().radius_tokens().lg)
            .bg(color.opacity(0.12))
            .text_color(color)
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
            .child(if playing {
                div().size(rems(0.5)).bg(color).into_any_element()
            } else {
                Icon::new(IconName::Play)
                    .with_size(px(12.))
                    .into_any_element()
            })
            .on_click(cx.listener(move |_, _, _, cx| {
                if playing {
                    cx.emit(MouseEditorEvent::StopPreviewRequested { file_id: Some(id) });
                } else {
                    cx.emit(MouseEditorEvent::PreviewRequested {
                        file_id: id,
                        path: path.clone(),
                    });
                }
            }));
        let status: SharedString = match &file.preview_state {
            PreviewState::Error(message) => message.clone(),
            _ => match file.state {
                UploadState::Uploading => "Uploading…".into(),
                UploadState::Failed => "Upload failed; remove and try again.".into(),
                _ => file.size.clone().unwrap_or_else(|| "Audio file".into()),
            },
        };
        h_flex()
            .id(format!("mouse-sound-row-{}-{id}", u8::from(catalog)))
            .w_full()
            .min_w_0()
            .items_center()
            .gap_2()
            .p_2()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius_tokens().lg)
            .bg(cx.theme().background)
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
                            .truncate()
                            .child(status),
                    ),
            )
            .when(ready, |row| {
                row.child(div().w(rems(12.)).min_w_0().flex_shrink_0().child(
                    render_audio_waveform(
                        format!("mouse-waveform-{}-{id}", u8::from(catalog)).into(),
                        self.audio_details.get(&id),
                        playing,
                        color,
                        cx,
                    ),
                ))
            })
            .child(if catalog {
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(self.render_sound_checkbox(&file, cx))
                    .child(self.render_delete_sound(&file, cx))
                    .into_any_element()
            } else {
                Button::new(format!("mouse-unassign-{id}"))
                    .ghost()
                    .small()
                    .icon(IconName::Close)
                    .accessibility_label(format!("Unassign {} from {}", file.name, button.label()))
                    .tooltip("Unassign from this button only")
                    .disabled(!assigned)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_file_assignment(id, button, CheckboxState::Unchecked, cx);
                    }))
                    .into_any_element()
            })
    }

    fn render_sound_checkbox(&self, file: &AudioFile, cx: &mut Context<Self>) -> impl IntoElement {
        let id = file.id;
        let button = self.selected_button;
        let state = if file.assigned_buttons[button.index()] {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        };
        let disabled = !matches!(file.state, UploadState::Success | UploadState::Complete);
        let entity = cx.entity().downgrade();
        Checkbox::new(format!("mouse-assign-{id}"))
            .state(state)
            .disabled(disabled)
            .accessibility_label(format!("Assign {} to {}", file.name, button.label()))
            .on_change(move |state, _, _, cx| {
                _ = entity.update(cx, |view, cx| {
                    view.toggle_file_assignment(id, button, state, cx)
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
                    .border_color(cx.theme().input)
                    .rounded(cx.theme().radius_tokens().sm)
                    .when(state == CheckboxState::Checked, |indicator| {
                        indicator
                            .bg(cx.theme().primary)
                            .border_color(cx.theme().primary)
                            .child(
                                Icon::new(IconName::Check)
                                    .with_size(px(12.))
                                    .text_color(cx.theme().primary_foreground),
                            )
                    }),
            )
    }

    fn render_delete_sound(&self, file: &AudioFile, cx: &mut Context<Self>) -> AnyElement {
        let id = file.id;
        let trigger = Button::new(format!("mouse-delete-sound-{id}"))
            .danger()
            .outline()
            .small()
            .icon(IconName::Trash)
            .accessibility_label(format!("Delete {} from the entire preset", file.name))
            .tooltip("Delete sound from the entire preset");
        if !file.assigned_buttons.iter().any(|assigned| *assigned) {
            return trigger
                .on_click(cx.listener(move |this, _, _, cx| this.remove_file(id, cx)))
                .into_any_element();
        }
        let entity = cx.entity().downgrade();
        let name = file.name.clone();
        AlertDialog::new(cx).trigger(trigger)
            .on_ok(move |_, _, cx| {
                _ = entity.update(cx, |view, cx| view.remove_file(id, cx));
                true
            })
            .content(move |content, _, _| {
                content.child(DialogHeader::new()
                    .child(DialogTitle::new().child("Delete sound from preset?"))
                    .child(DialogDescription::new().child(format!(
                        "{name} will be removed from all mouse buttons. Use the row's X to unassign it from only the selected button.",
                    ))))
                    .child(DialogFooter::new()
                        .child(DialogClose::new().child(Button::new(format!("mouse-cancel-delete-{id}")).outline().label("Keep sound")))
                        .child(DialogAction::new().child(Button::new(format!("mouse-confirm-delete-{id}")).danger().label("Delete sound"))))
            }).into_any_element()
    }

    pub(super) fn render_upload_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("mouse-audio-drop-zone")
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
            .tooltip("Choose audio files or drop them here, then assign them in the sound catalog")
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                this.begin_uploads(paths.paths().to_vec(), window, cx);
            }))
            .on_click(cx.listener(|this, _, window, cx| this.open_file_picker(window, cx)))
    }
}
