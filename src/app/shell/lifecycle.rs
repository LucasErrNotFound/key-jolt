use super::super::title_bar::{AppTitleBar, AppTitleBarEvent};

use super::{AppPage, AppShell};

use crate::audio::playback::PlaybackHandle;
use crate::features::home::{HomeEvent, HomeView};

use crate::input;
use crate::platform::tray;
use crate::platform::window::platform_notice;
use crate::presets::PresetKind;
use crate::runtime::start_runtime_thread;
use crate::settings::appearance;
use crate::settings::appearance::{
    AppearanceMode, AppearanceSelection, ThemeDescriptor, resolve_selection,
};

use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};

impl AppShell {
    pub(in crate::app) fn new(
        home: Entity<HomeView>,
        tray_available: Arc<AtomicBool>,
        themes: Vec<ThemeDescriptor>,
        appearance_error: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let data_dir = home.read(cx).data_dir();
        let mut settings = home.read(cx).settings();
        let selection = resolve_selection(
            settings.appearance_mode,
            Some(&settings.appearance_theme_id),
            &themes,
        )
        .or_else(|| resolve_selection(settings.appearance_mode, None, &themes))
        .unwrap_or(AppearanceSelection {
            mode: AppearanceMode::Light,
            theme_id: appearance::default_theme_id(AppearanceMode::Light).to_string(),
        });
        settings.appearance_mode = selection.mode;
        settings.appearance_theme_id = selection.theme_id.clone();
        let title_bar = cx.new(|cx| {
            AppTitleBar::new(
                themes.clone(),
                selection,
                settings.run_at_startup,
                appearance_error,
                cx,
            )
        });
        let title_bar_subscription = cx.subscribe_in(
            &title_bar,
            window,
            |this, _, event: &AppTitleBarEvent, window, cx| {
                this.on_title_bar_event(event, window, cx)
            },
        );
        let settings_generation = Arc::new(AtomicU64::new(0));
        let settings_writer = Arc::new(Mutex::new(super::settings_persistence::SettingsWriter {
            run_at_startup: settings.run_at_startup,
        }));
        let (runtime_tx, runtime_rx) = mpsc::channel();
        let mut startup_notifications = Vec::new();
        let tray_icon = match tray::create(runtime_tx.clone(), cx) {
            Ok(icon) => {
                tray_available.store(true, Ordering::Release);
                Some(icon)
            }
            Err(error) => {
                startup_notifications.push(
                    Notification::warning(format!("System tray unavailable: {error}"))
                        .title("System tray")
                        .placement(Anchor::BottomRight)
                        .autohide(true)
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.notify();
                            cx.hide();
                        })),
                );
                None
            }
        };
        let (status_tx, status_rx) = mpsc::channel();
        let playback = PlaybackHandle::new(data_dir.clone(), &settings, status_tx.clone());
        if let Some(id) = settings.active_keyboard_preset_id.as_ref() {
            playback.reload(PresetKind::Keyboard, id.clone());
        }
        if let Some(id) = settings.active_mouse_preset_id.as_ref() {
            playback.reload(PresetKind::Mouse, id.clone());
        }
        let listener = input::start_listener();
        let runtime_enabled = playback.app_enabled.clone();
        let runtime_playback = playback.sender_handle();
        let runtime_thread = start_runtime_thread(
            listener,
            runtime_playback,
            runtime_enabled,
            runtime_tx,
            status_rx,
        );
        let startup_warnings = home.read(cx).startup_warnings().to_vec();
        for message in startup_warnings {
            startup_notifications.push(
                Notification::warning(message)
                    .title("Preset or settings warning")
                    .placement(Anchor::BottomRight)
                    .autohide(true)
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.notify();
                        cx.hide();
                    })),
            );
        }
        if let Some(message) = platform_notice() {
            startup_notifications.push(
                Notification::warning(message)
                    .title("Global input access")
                    .placement(Anchor::BottomRight)
                    .autohide(false)
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.notify();
                        cx.hide();
                    })),
            );
        }

        if !startup_notifications.is_empty() {
            cx.defer_in(window, move |_, window, cx| {
                for notification in startup_notifications {
                    window.push_notification(notification, cx);
                }
            });
        }

        let home_subscription =
            cx.subscribe_in(&home, window, |this, _, event: &HomeEvent, window, cx| {
                this.on_home_event(event, window, cx)
            });

        let runtime_task = Self::start_runtime_events(runtime_rx, window, cx);

        Self {
            title_bar,
            _title_bar_subscription: title_bar_subscription,
            home,
            page: AppPage::Home,
            _home_subscription: home_subscription,
            editor_subscription: None,
            active_preview: None,
            next_preview_id: 0,
            data_dir,
            settings,
            _save_task: None,
            _delete_task: None,
            _settings_task: None,
            settings_generation,
            settings_writer,
            startup_change_pending: false,
            themes,
            playback,
            _tray_icon: tray_icon,
            _runtime_thread: runtime_thread,
            _runtime_task: runtime_task,
        }
    }
}
