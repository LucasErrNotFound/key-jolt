use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use super::home_view::{MousePresetData, MouseSoundData};
use gpui_kit::assets::IconName;
use gpui_kit::base::{Checkbox, CheckboxIndicator, CheckboxState};
use gpui_kit::component::WindowExt;
use gpui_kit::component::attachment::{
    Attachment, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentMedia,
    AttachmentStatus, AttachmentTitle,
};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::{
    AlertDialog, DialogAction, DialogClose, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle,
};
use gpui_kit::component::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::label::Label;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

#[derive(Clone, Debug)]
pub enum MouseEditorEvent {
    BackRequested,
    PreviewRequested {
        file_id: u64,
        path: PathBuf,
    },
    StopPreviewRequested {
        file_id: Option<u64>,
    },
    PlaybackStateChanged {
        state: PreviewState,
    },
    SaveRequested {
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        summary: SharedString,
        data: MousePresetData,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewState {
    Stopped,
    Playing,
    Finished,
    Error(SharedString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveStatus {
    Idle,
    Saving,
    Failed(SharedString),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Middle => 2,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Left => "Left click",
            Self::Right => "Right click",
            Self::Middle => "Middle · scroll wheel",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UploadState {
    Uploading,
    Success,
    Complete,
    Failed,
}

#[derive(Clone)]
struct AudioFile {
    id: u64,
    path: PathBuf,
    name: SharedString,
    size: Option<SharedString>,
    assigned_buttons: [bool; 3],
    state: UploadState,
    preview_state: PreviewState,
}

impl AudioFile {
    fn new(id: u64, path: PathBuf, name: SharedString) -> Self {
        Self {
            id,
            path,
            name,
            size: None,
            assigned_buttons: [false; 3],
            state: UploadState::Uploading,
            preview_state: PreviewState::Stopped,
        }
    }
}

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "oga", "opus", "m4a", "aac", "aiff", "aif", "wma", "webm", "ac3",
    "amr", "mid", "midi",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlaybackMode {
    Sequential,
    Random,
}

pub struct MouseEditorView {
    preset_id: Option<SharedString>,
    preset_name: SharedString,
    name_input: Entity<InputState>,
    _name_subscription: Subscription,
    initial_data: MousePresetData,
    selected_button: MouseButton,
    files: Vec<AudioFile>,
    next_file_id: u64,
    upload_tasks: HashMap<u64, Task<()>>,
    hovered_file: Option<u64>,
    playback_mode: PlaybackMode,
    save_status: SaveStatus,
}

impl EventEmitter<MouseEditorEvent> for MouseEditorView {}

impl MouseEditorView {
    pub fn view(
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<MousePresetData>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(preset_id, preset_name, preset_data, window, cx))
    }

    pub fn set_preview_state(&mut self, file_id: u64, state: PreviewState, cx: &mut Context<Self>) {
        if let Some(file) = self.files.iter_mut().find(|file| file.id == file_id) {
            file.preview_state = state.clone();
            cx.emit(MouseEditorEvent::PlaybackStateChanged { state });
            cx.notify();
        }
    }

    pub fn set_save_status(&mut self, status: SaveStatus, cx: &mut Context<Self>) {
        self.save_status = status;
        cx.notify();
    }

    fn new(
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<MousePresetData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Preset name")
                .default_value(preset_name.as_ref())
        });
        let name_subscription =
            cx.subscribe_in(&name_input, window, |_, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            });
        let mut initial_data = preset_data.unwrap_or(MousePresetData {
            files: Vec::new(),
            random_playback: false,
        });
        let assigned_sound_count = initial_data
            .files
            .iter()
            .filter(|file| file.assigned_buttons[MouseButton::Left.index()])
            .count();
        if assigned_sound_count < 2 {
            initial_data.random_playback = false;
        }
        let files = initial_data
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| AudioFile {
                id: index as u64,
                path: file.path.clone(),
                name: file.name.clone(),
                size: file.size.clone(),
                assigned_buttons: file.assigned_buttons,
                state: UploadState::Complete,
                preview_state: PreviewState::Stopped,
            })
            .collect::<Vec<_>>();
        let playback_mode = if initial_data.random_playback {
            PlaybackMode::Random
        } else {
            PlaybackMode::Sequential
        };
        if assigned_sound_count < 2 {
            println!("Mouse playback mode: None");
        } else {
            println!(
                "Mouse playback mode: {}",
                match playback_mode {
                    PlaybackMode::Sequential => "Sequential",
                    PlaybackMode::Random => "Random",
                }
            );
        }
        let next_file_id = files.len() as u64;

        Self {
            preset_id,
            preset_name,
            name_input,
            _name_subscription: name_subscription,
            initial_data,
            selected_button: MouseButton::Left,
            files,
            next_file_id,
            upload_tasks: HashMap::new(),
            hovered_file: None,
            playback_mode,
            save_status: SaveStatus::Idle,
        }
    }

    fn select_button(&mut self, button: MouseButton, cx: &mut Context<Self>) {
        if self.selected_button != button {
            let previous_count = self.assigned_sound_count();
            self.selected_button = button;
            self.update_playback_mode_for_selection_change(previous_count);
            cx.notify();
        }
    }

    fn set_playback_mode(&mut self, mode: PlaybackMode, cx: &mut Context<Self>) {
        if mode == PlaybackMode::Random && !self.random_playback_available() {
            return;
        }

        self.playback_mode = mode;
        println!(
            "Mouse playback mode: {}",
            match mode {
                PlaybackMode::Sequential => "Sequential",
                PlaybackMode::Random => "Random",
            }
        );
        cx.notify();
    }

    fn random_playback_available(&self) -> bool {
        self.assigned_sound_count() >= 2
    }

    fn assigned_sound_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.assigned_buttons[self.selected_button.index()])
            .count()
    }

    fn update_playback_mode_for_selection_change(&mut self, previous_count: usize) {
        let current_count = self.assigned_sound_count();
        if current_count == previous_count {
            return;
        }

        if current_count < 2 {
            self.playback_mode = PlaybackMode::Sequential;
            println!("Mouse playback mode: None");
        } else if previous_count < 2 {
            self.playback_mode = PlaybackMode::Sequential;
            println!("Mouse playback mode: Sequential");
        }
    }

    fn has_unsaved_changes(&self, cx: &App) -> bool {
        self.current_preset_data() != self.initial_data
            || self.name_input.read(cx).value() != self.preset_name.as_ref()
    }

    fn current_preset_data(&self) -> MousePresetData {
        MousePresetData {
            files: self
                .files
                .iter()
                .map(|file| MouseSoundData {
                    path: file.path.clone(),
                    name: file.name.clone(),
                    size: file.size.clone(),
                    assigned_buttons: file.assigned_buttons,
                })
                .collect(),
            random_playback: self.playback_mode == PlaybackMode::Random,
        }
    }

    fn save_validation_error(&self, name: &str) -> Option<SharedString> {
        if let Err(message) = crate::storage::validate_preset_name(name) {
            return Some(message.into());
        }

        if !self.files.iter().any(|file| {
            file.assigned_buttons.iter().any(|assigned| *assigned)
                && matches!(file.state, UploadState::Success | UploadState::Complete)
        }) {
            return Some(
                "Assign at least one successfully uploaded sound to a mouse button.".into(),
            );
        }

        None
    }

    fn format_file_size(bytes: u64) -> SharedString {
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

    fn is_audio_file(path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                AUDIO_EXTENSIONS
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(extension))
            })
            .unwrap_or(false)
    }

    fn begin_uploads(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        for path in paths {
            if !Self::is_audio_file(&path) {
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
                println!("Duplicate mouse sound rejected: {name}");
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

    fn open_file_picker(&self, window: &mut Window, cx: &mut Context<Self>) {
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let files = rfd::AsyncFileDialog::new()
                .set_title("Select mouse sounds")
                .add_filter("Audio files", AUDIO_EXTENSIONS)
                .pick_files()
                .await;

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

    fn remove_file(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(index) = self.files.iter().position(|file| file.id == id) {
            let previous_count = self.assigned_sound_count();
            if self.files[index].preview_state == PreviewState::Playing {
                cx.emit(MouseEditorEvent::StopPreviewRequested { file_id: Some(id) });
            }
            let file = self.files.remove(index);
            self.upload_tasks.remove(&id);
            if self.hovered_file == Some(id) {
                self.hovered_file = None;
            }
            self.update_playback_mode_for_selection_change(previous_count);
            println!("Mouse sound removed: {}", file.name);
            cx.notify();
        }
    }

    fn toggle_file_assignment(
        &mut self,
        id: u64,
        button: MouseButton,
        state: CheckboxState,
        cx: &mut Context<Self>,
    ) {
        let previous_count = self.assigned_sound_count();
        if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
            let assigned = state == CheckboxState::Checked;
            file.assigned_buttons[button.index()] = assigned;
            println!(
                "{} {} {}",
                file.name,
                if assigned {
                    "assigned to"
                } else {
                    "removed from"
                },
                button.label()
            );
        }
        self.update_playback_mode_for_selection_change(previous_count);
        cx.notify();
    }

    fn render_mouse_button(
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
                cx.theme().blue
            } else {
                cx.theme().muted_foreground.opacity(0.25)
            })
            .when(is_selected, |this| this.bg(cx.theme().blue.opacity(0.07)))
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

    fn render_mouse_diagram(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let left_active = self.selected_button == MouseButton::Left;
        let right_active = self.selected_button == MouseButton::Right;
        let middle_active = self.selected_button == MouseButton::Middle;
        let entity = cx.entity().downgrade();
        let left_entity = entity.clone();
        let right_entity = entity.clone();
        let middle_entity = entity;
        let highlight_color = cx.theme().blue.opacity(0.14);

        v_flex()
            .relative()
            .w(px(180.))
            .h(px(270.))
            .rounded_full()
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.35))
            .overflow_hidden()
            .child(
                canvas(
                    |bounds, _, _| bounds,
                    move |_, bounds, window, _| {
                        let origin = bounds.origin;
                        let top = origin.y + px(1.);
                        let left = origin.x + px(1.);
                        let right = origin.x + bounds.size.width - px(1.);
                        let center = origin.x + bounds.size.width / 2.;
                        let radius = bounds.size.width / 2. - px(1.);
                        let control_factor = 0.552_284_8;
                        let divider_left = center - px(29.) / 2.;
                        let divider_right = center + px(29.) / 2.;
                        let button_bottom = top + px(115.);

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
                    .mx(px(1.))
                    .mt(px(1.))
                    .h(px(115.))
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
                            .w(px(29.))
                            .h_full()
                            .items_center()
                            .justify_center()
                            .border_x_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.25))
                            .bg(cx.theme().muted.opacity(0.12))
                            .child(
                                div()
                                    .id("mouse-middle-wheel")
                                    .w(px(14.))
                                    .h(px(36.))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(if middle_active {
                                        cx.theme().blue
                                    } else {
                                        cx.theme().muted_foreground.opacity(0.45)
                                    })
                                    .bg(if middle_active {
                                        cx.theme().blue.opacity(0.22)
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
                    .mx(px(1.))
                    .mb(px(1.))
                    .flex_1()
                    .border_t_1()
                    .border_color(cx.theme().muted_foreground.opacity(0.25))
                    .bg(cx.theme().muted.opacity(0.12)),
            )
    }

    fn render_file_row(
        &self,
        file: AudioFile,
        button: MouseButton,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = file.id;
        let checked = file.assigned_buttons[button.index()];
        let hovered = self.hovered_file == Some(id);
        let entity = cx.entity().downgrade();

        let media_content = match file.state {
            UploadState::Uploading => Spinner::new()
                .small()
                .color(cx.theme().blue)
                .into_any_element(),
            UploadState::Success => Icon::new(IconName::CircleCheck)
                .with_size(px(18.))
                .text_color(cx.theme().green)
                .into_any_element(),
            UploadState::Complete => {
                let preview_entity = entity.clone();
                let path = file.path.clone();
                let is_playing = file.preview_state == PreviewState::Playing;
                let preview_button = Button::new(format!("preview-mouse-file-{id}"))
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
                    .on_click(move |_, _, cx| {
                        _ = preview_entity.update(cx, |_, cx| {
                            if is_playing {
                                cx.emit(MouseEditorEvent::StopPreviewRequested {
                                    file_id: Some(id),
                                });
                            } else {
                                cx.emit(MouseEditorEvent::PreviewRequested {
                                    file_id: id,
                                    path: path.clone(),
                                });
                            }
                        });
                    });

                div()
                    .id(format!("preview-hover-mouse-file-{id}"))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_hover({
                        let entity = entity.clone();
                        move |is_hovered, _, cx| {
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
                .with_size(px(18.))
                .text_color(cx.theme().red)
                .into_any_element(),
        };

        let status = match file.state {
            UploadState::Uploading => AttachmentStatus::Uploading,
            UploadState::Success | UploadState::Complete => AttachmentStatus::Complete,
            UploadState::Failed => AttachmentStatus::Failed,
        };
        let description = match file.state {
            UploadState::Uploading => "Checking file…".into(),
            UploadState::Success | UploadState::Complete => match &file.preview_state {
                PreviewState::Error(message) => message.clone(),
                _ => file.size.clone().unwrap_or_else(|| "Preparing…".into()),
            },
            UploadState::Failed => "Could not read file".into(),
        };
        let remove_entity = cx.entity().downgrade();
        let checkbox_entity = cx.entity().downgrade();

        let attachment = Attachment::new()
            .small()
            .status(status)
            .flex_1()
            .media(AttachmentMedia::new().child(media_content))
            .content(
                AttachmentContent::new()
                    .title(AttachmentTitle::new(file.name.clone()))
                    .description(AttachmentDescription::new(description)),
            )
            .actions(
                AttachmentActions::new().child(
                    Button::new(format!("remove-mouse-file-{id}"))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Close)
                        .accessibility_label(format!("Remove {}", file.name))
                        .tooltip("Remove")
                        .on_click(move |_, _, cx| {
                            _ = remove_entity.update(cx, |view, cx| view.remove_file(id, cx));
                        }),
                ),
            );

        let checkbox = Checkbox::new(format!("assign-mouse-file-{id}-{}", button.index()))
            .checked(checked)
            .accessibility_label(format!("Assign {} to {}", file.name, button.label()))
            .on_change(move |state, _, _, cx| {
                _ = checkbox_entity.update(cx, |view, cx| {
                    view.toggle_file_assignment(id, button, state, cx);
                });
            })
            .child(
                CheckboxIndicator::new()
                    .checked(checked)
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_4()
                    .border_1()
                    .border_color(cx.theme().muted_foreground.opacity(0.4))
                    .when(checked, |this| {
                        this.bg(cx.theme().blue)
                            .border_color(cx.theme().blue)
                            .child(
                                Icon::new(IconName::Check)
                                    .with_size(px(12.))
                                    .text_color(cx.theme().primary_foreground),
                            )
                    }),
            );

        h_flex()
            .items_center()
            .gap_3()
            .w_full()
            .child(attachment)
            .child(
                div()
                    .w(px(28.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(checkbox),
            )
    }

    fn render_upload_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Empty::new()
            .w_full()
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.25))
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Icon)
                            .child(Icon::new(IconName::Upload).with_size(px(22.))),
                    )
                    .title(EmptyTitle::new().child("Add audio files"))
                    .description(
                        EmptyDescription::new().child("Select one or more sounds for this button."),
                    ),
            )
            .content(
                EmptyContent::new().flex_row().justify_center().child(
                    Button::new("upload-mouse-files")
                        .primary()
                        .label("Upload Files…")
                        .on_click(
                            cx.listener(|this, _, window, cx| this.open_file_picker(window, cx)),
                        ),
                ),
            )
    }

    fn render_playback(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let random_playback_available = self.random_playback_available();
        h_flex()
            .gap_1()
            .child(
                Button::new("mouse-playback-sequential")
                    .disabled(!random_playback_available)
                    .label("Sequential")
                    .when(
                        random_playback_available && self.playback_mode == PlaybackMode::Sequential,
                        |this| this.primary(),
                    )
                    .when(
                        random_playback_available && self.playback_mode != PlaybackMode::Sequential,
                        |this| this.outline(),
                    )
                    .when(!random_playback_available, |this| {
                        this.ghost().text_color(cx.theme().muted_foreground)
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.set_playback_mode(PlaybackMode::Sequential, cx)
                    })),
            )
            .child(
                Button::new("mouse-playback-random")
                    .disabled(!random_playback_available)
                    .label("Random")
                    .when(
                        random_playback_available && self.playback_mode == PlaybackMode::Random,
                        |this| this.primary(),
                    )
                    .when(
                        random_playback_available && self.playback_mode != PlaybackMode::Random,
                        |this| this.outline(),
                    )
                    .when(!random_playback_available, |this| {
                        this.ghost().text_color(cx.theme().muted_foreground)
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.set_playback_mode(PlaybackMode::Random, cx)
                    })),
            )
    }
}

impl Render for MouseEditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button = self.selected_button;
        let files = self.files.clone();
        let selected_label = button.label();
        let file_count = self
            .files
            .iter()
            .filter(|file| file.assigned_buttons[button.index()])
            .count();
        let preset_id = self.preset_id.clone();
        let creating = preset_id.is_none();
        let name_input = self.name_input.clone();
        let has_unsaved_changes = self.has_unsaved_changes(cx);
        let mut file_rows = Vec::with_capacity(files.len());
        for file in files {
            file_rows.push(self.render_file_row(file, button, cx).into_any_element());
        }
        let back_entity = cx.entity().downgrade();
        let back_control = if has_unsaved_changes {
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
                                        .with_size(px(24.))
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
        };
        let editor_identity = if creating {
            v_flex()
                .gap_1()
                .child(Label::new("Creating mouse preset").text_color(cx.theme().muted_foreground))
                .into_any_element()
        } else {
            h_flex()
                .gap_1()
                .child(Label::new("Editing").text_color(cx.theme().muted_foreground))
                .child(Label::new(self.preset_name.clone()))
                .into_any_element()
        };

        v_flex().size_full().font_family("Segoe UI").child(
            div().flex_1().min_h_0().overflow_y_scrollbar().child(
                v_flex()
                    .px_6()
                    .py_4()
                    .gap_4()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .child(
                                h_flex().items_center().gap_3().child(back_control).child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            Label::new("Mouse configuration")
                                                .font_semibold()
                                                .text_size(px(17.)),
                                        )
                                        .child(editor_identity),
                                ),
                            )
                            .child(
                                Button::new("save-mouse-preset")
                                    .primary()
                                    .label(match &self.save_status {
                                        SaveStatus::Idle => "Save preset",
                                        SaveStatus::Saving => "Saving…",
                                        SaveStatus::Failed(_) => "Retry save",
                                    })
                                    .disabled(
                                        matches!(self.save_status, SaveStatus::Saving)
                                            || !self.has_unsaved_changes(cx),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if matches!(this.save_status, SaveStatus::Saving)
                                            || !this.has_unsaved_changes(cx)
                                        {
                                            return;
                                        }
                                        let preset_name =
                                            name_input.read(cx).value().trim().to_string();
                                        if let Some(message) =
                                            this.save_validation_error(&preset_name)
                                        {
                                            window.push_notification(
                                                Notification::warning(message)
                                                    .title("Cannot save preset")
                                                    .placement(Anchor::BottomRight)
                                                    .autohide(true)
                                                    .on_click(cx.listener(|_, _, _, cx| {
                                                        cx.notify();
                                                        cx.hide();
                                                    })),
                                                cx,
                                            );
                                            return;
                                        }
                                        let mapped_button_count = [
                                            MouseButton::Left,
                                            MouseButton::Right,
                                            MouseButton::Middle,
                                        ]
                                        .iter()
                                        .filter(|button| {
                                            this.files.iter().any(|file| {
                                                file.assigned_buttons[button.index()]
                                                    && matches!(
                                                        file.state,
                                                        UploadState::Success
                                                            | UploadState::Complete
                                                    )
                                            })
                                        })
                                        .count();
                                        let mapped_button_label = if mapped_button_count == 1 {
                                            "button"
                                        } else {
                                            "buttons"
                                        };
                                        cx.emit(MouseEditorEvent::SaveRequested {
                                            preset_id: preset_id.clone(),
                                            preset_name: preset_name.into(),
                                            summary: format!(
                                                "Custom · {mapped_button_count} {mapped_button_label} mapped"
                                            )
                                            .into(),
                                            data: this.current_preset_data(),
                                        });
                                    })),
                            ),
                    )
                    .when(creating, |this| {
                        this.child(
                            v_flex()
                                .w_full()
                                .gap_1()
                                .child(Label::new("Preset name").text_sm())
                                .child(
                                    Input::new(&self.name_input)
                                        .w(px(280.))
                                        .id("mouse-preset-name")
                                        .cleanable(true)
                                        .aria_label("Preset name"),
                                ),
                        )
                    })
                    .child(Separator::horizontal())
                    .child(
                        h_flex()
                            .items_center()
                            .justify_center()
                            .gap_8()
                            .py_2()
                            .child(self.render_mouse_diagram(cx))
                            .child(
                                v_flex()
                                    .w(px(220.))
                                    .gap_2()
                                    .child(self.render_mouse_button(
                                        MouseButton::Left,
                                        "Left click",
                                        cx,
                                    ))
                                    .child(self.render_mouse_button(
                                        MouseButton::Right,
                                        "Right click",
                                        cx,
                                    ))
                                    .child(self.render_mouse_button(
                                        MouseButton::Middle,
                                        "Middle · scroll",
                                        cx,
                                    )),
                            ),
                    )
                    .child(Separator::horizontal())
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                Label::new(format!("Editing {selected_label}"))
                                    .font_semibold()
                                    .text_size(px(18.)),
                            )
                            .child(
                                Label::new(if file_count == 0 {
                                    "Assign one sound, or several for a random pool.".to_string()
                                } else {
                                    format!(
                                        "{file_count} sound{} assigned to {selected_label}.",
                                        if file_count == 1 { "" } else { "s" }
                                    )
                                })
                                .text_sm()
                                .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .children(file_rows)
                            .child(self.render_upload_area(cx)),
                    )
                    .child(Separator::horizontal())
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Label::new("Playback")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(self.render_playback(cx)),
                    )
                    .child(
                        Label::new(match self.playback_mode {
                            PlaybackMode::Sequential => {
                                "Sounds play in order, cycling back to the first."
                            }
                            PlaybackMode::Random => {
                                "Sounds are selected randomly from this button’s pool."
                            }
                        })
                        .text_sm()
                        .text_color(if self.random_playback_available() {
                            cx.theme().foreground
                        } else {
                            cx.theme().muted_foreground
                        }),
                    ),
            ),
        )
    }
}
