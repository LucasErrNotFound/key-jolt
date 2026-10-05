use super::{MouseEditorEvent, MouseEditorView};

use super::model::AudioFile;
use crate::features::preset_editor::{PreviewState, UploadState};

use crate::features::preset_editor::{AUDIO_EXTENSIONS, is_audio_file};
use crate::platform::file_dialog::begin_file_dialog;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;

use gpui_kit::*;
use std::path::PathBuf;
use std::time::Duration;

impl MouseEditorView {
    pub(super) fn format_file_size(bytes: u64) -> SharedString {
        const KB: f64 = 1024.0;
        const MB: f64 = 1024.0 * KB;
        if bytes >= MB as u64 {
            format!("{:.1} MB", bytes as f64 / MB).into()
        } else if bytes >= KB as u64 {
            format!("{:.0} KB", bytes as f64 / KB).into()
        } else {
            format!("{bytes} B").into()
        }
    }

    pub(super) fn begin_uploads(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for path in paths {
            if !is_audio_file(&path) {
                continue;
            }

            let Some(name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(SharedString::from)
            else {
                continue;
            };

            if self.files.iter().any(|file| file.name == name) {
                window.push_notification(
                    Notification::error(format!("\"{}\" is already in the sound list.", name))
                        .title("Duplicate sound")
                        .placement(Anchor::BottomRight)
                        .autohide(true)
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.notify();
                            cx.hide();
                        })),
                    cx,
                );
                continue;
            }

            let id = self.next_file_id;
            self.next_file_id += 1;
            self.files.push(AudioFile::new(id, path.clone(), name));

            let task = cx.spawn(async move |this, cx| {
                let metadata = cx
                    .background_spawn(async move {
                        std::fs::metadata(&path).map(|metadata| metadata.len())
                    })
                    .await;

                let Ok(size) = metadata else {
                    _ = this.update(cx, |view, cx| {
                        if let Some(file) = view.files.iter_mut().find(|file| file.id == id) {
                            file.state = UploadState::Failed;
                        }
                        cx.notify();
                    });
                    return;
                };

                if this
                    .update(cx, |view, cx| {
                        if let Some(file) = view.files.iter_mut().find(|file| file.id == id) {
                            file.size = Some(Self::format_file_size(size));
                            file.state = UploadState::Success;
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }

                cx.background_executor()
                    .timer(Duration::from_millis(450))
                    .await;
                _ = this.update(cx, |view, cx| {
                    if let Some(file) = view.files.iter_mut().find(|file| file.id == id) {
                        file.state = UploadState::Complete;
                    }
                    cx.notify();
                });
            });
            self.upload_tasks.insert(id, task);
        }

        cx.notify();
    }

    pub(super) fn open_file_picker(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((dialog_guard, dialog)) = begin_file_dialog(window) else {
            return;
        };
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let files = dialog
                .set_title("Select mouse sounds")
                .add_filter("Audio files", AUDIO_EXTENSIONS)
                .pick_files()
                .await;
            drop(dialog_guard);

            let Some(files) = files else {
                return;
            };
            let paths: Vec<PathBuf> = files
                .into_iter()
                .map(|file| file.path().to_path_buf())
                .collect();
            _ = window_handle.update(cx, |_, window, cx| {
                _ = this.update(cx, |view, cx| {
                    if !paths.is_empty() {
                        cx.emit(MouseEditorEvent::StopPreviewRequested { file_id: None });
                    }
                    view.begin_uploads(paths, window, cx);
                });
            });
        })
        .detach();
    }

    pub(super) fn remove_file(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(index) = self.files.iter().position(|file| file.id == id) {
            let previous_count = self.assigned_sound_count();
            if self.files[index].preview_state == PreviewState::Playing {
                cx.emit(MouseEditorEvent::StopPreviewRequested { file_id: Some(id) });
            }
            self.files.remove(index);
            self.upload_tasks.remove(&id);
            if self.hovered_file == Some(id) {
                self.hovered_file = None;
            }
            self.update_playback_mode_for_selection_change(previous_count);
            cx.notify();
        }
    }
}
