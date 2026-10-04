use gpui_kit::SharedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PreviewState {
    Stopped,
    Playing,
    Finished,
    Error(SharedString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SaveStatus {
    Idle,
    Saving,
    Failed(SharedString),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum UploadState {
    Uploading,
    Success,
    Complete,
    Failed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlaybackMode {
    Sequential,
    Random,
}

pub(crate) const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "oga", "opus", "m4a", "aac", "aiff", "aif", "wma", "webm", "ac3",
    "amr", "mid", "midi",
];
pub(crate) fn is_audio_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            AUDIO_EXTENSIONS
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(extension))
        })
}
