mod events;
mod lifecycle;
mod navigation;
mod presets;
mod preview;
mod runtime;
mod settings;
mod settings_persistence;

use super::title_bar::AppTitleBar;
use crate::audio::playback::PlaybackHandle;
use crate::features::home::HomeView;
use crate::features::keyboard::KeyboardEditorView;
use crate::features::mouse::MouseEditorView;
use crate::features::preset_editor::PreviewState;
use crate::presets::PresetKind;
use crate::settings::appearance::ThemeDescriptor;
use crate::settings::store::AppSettings;
use gpui_kit::base::v_flex;
use gpui_kit::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use std::thread::JoinHandle;

pub(super) struct AppShell {
    title_bar: Entity<AppTitleBar>,
    _title_bar_subscription: Subscription,
    home: Entity<HomeView>,
    page: AppPage,
    _home_subscription: Subscription,
    editor_subscription: Option<Subscription>,
    active_preview: Option<ActivePreview>,
    next_preview_id: u64,
    data_dir: PathBuf,
    settings: AppSettings,
    _save_task: Option<Task<()>>,
    _delete_task: Option<Task<()>>,
    _settings_task: Option<Task<()>>,
    settings_generation: Arc<AtomicU64>,
    settings_writer: Arc<Mutex<settings_persistence::SettingsWriter>>,
    startup_change_pending: bool,
    themes: Vec<ThemeDescriptor>,
    playback: PlaybackHandle,
    _tray_icon: Option<tray_icon::TrayIcon>,
    _runtime_thread: JoinHandle<()>,
    _runtime_task: Task<()>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum AppShellEvent {
    SettingsChanged(AppSettings),
    PresetReloadRequested { kind: PresetKind, id: SharedString },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PreviewTarget {
    Keyboard(u64),
    Mouse(u64),
}

struct ActivePreview {
    id: u64,
    target: PreviewTarget,
    stop_requested: Arc<AtomicBool>,
    _task: Task<()>,
}

enum AppPage {
    Home,
    KeyboardEditor(Entity<KeyboardEditorView>),
    MouseEditor(Entity<MouseEditorView>),
}

impl EventEmitter<AppShellEvent> for AppShell {}

impl Drop for AppShell {
    fn drop(&mut self) {
        if let Some(active) = self.active_preview.as_ref() {
            active.stop_requested.store(true, Ordering::Release);
        }
    }
}

impl Render for AppShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content = match &self.page {
            AppPage::Home => self.home.clone().into_any_element(),
            AppPage::KeyboardEditor(editor) => editor.clone().into_any_element(),
            AppPage::MouseEditor(editor) => editor.clone().into_any_element(),
        };

        v_flex()
            .size_full()
            .child(self.title_bar.clone())
            .child(div().flex_1().min_h_0().child(content))
    }
}
