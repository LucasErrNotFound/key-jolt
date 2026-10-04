use crate::settings::store::AppSettings;
use gpui_kit::*;

#[derive(Clone, Debug)]
pub(crate) enum HomeEvent {
    ImportKeyboardRequested,
    CreateKeyboardRequested,
    EditRequested(SharedString),
    DeleteKeyboardRequested(SharedString),
    ImportMousesRequested,
    CreateMouseRequested,
    ConfigureMouseRequested(SharedString),
    DeleteMouseRequested(SharedString),
    SettingsChanged {
        settings: AppSettings,
        persist: bool,
    },
}
