mod app_assets;
mod appearance;
mod audio_playback;
mod audio_preview;
mod input;
mod settings_store;
mod storage;
mod tray;
mod view;

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use app_assets::AppAssets;
use appearance::{
    AppearanceMode, AppearanceSelection, ThemeDescriptor, pair_selection, resolve_selection,
};
use audio_playback::PlaybackHandle;
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::button::ButtonVariants;
use gpui_kit::component::popover::Popover;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::theme::{Theme, ThemeMode, ThemeRegistry};
use gpui_kit::component::{ActiveTheme as _, Selectable as _, WindowExt};
use gpui_kit::component::{Root, TitleBar, button::Button, notification::Notification};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use input::{InputAction, InputState, Listener, Throttle};
use settings_store::AppSettings;
use storage::PresetKind;
use view::home_view::{HomeEvent, HomeView, KeyboardPresetData, MousePresetData};
use view::keyboard_editor::{KeyboardEditorEvent, KeyboardEditorView};
use view::mouse_editor::{MouseEditorEvent, MouseEditorView};

const GITHUB_REPOSITORY_URL: &str = "https://github.com/LucasErrNotFound/key-jolt";

struct AppTitleBar {
    appearance_picker: Entity<AppearancePicker>,
    popover_open: bool,
    _picker_subscription: Subscription,
}

struct AppearancePicker {
    focus_handle: FocusHandle,
    selection: AppearanceSelection,
    themes: Vec<ThemeDescriptor>,
    error_message: Option<String>,
}

#[derive(Clone, Debug)]
enum AppTitleBarEvent {
    ModeSelected(AppearanceMode),
    ThemeSelected(String),
}

impl EventEmitter<AppTitleBarEvent> for AppTitleBar {}

struct AppShell {
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
    themes: Vec<ThemeDescriptor>,
    playback: PlaybackHandle,
    _tray_icon: Option<tray_icon::TrayIcon>,
    _runtime_thread: JoinHandle<()>,
    _runtime_task: Task<()>,
}

enum RuntimeEvent {
    ToggleApp(bool),
    Warning(String),
    TrayOpen,
    TrayExit,
}

fn start_runtime_thread(
    listener: Listener,
    playback: audio_playback::PlaybackSender,
    enabled: Arc<AtomicBool>,
    events: Sender<RuntimeEvent>,
    audio_status: Receiver<String>,
) -> JoinHandle<()> {
    let thread_events = events.clone();
    std::thread::Builder::new()
        .name("key-jolt-input-controller".to_string())
        .spawn(move || {
            let Listener {
                events: input_events,
                status: listener_status,
                ..
            } = listener;
            let mut state = InputState::new();
            let mut mouse_throttle = Throttle::new(Duration::from_millis(12));
            let mut listener_running = true;
            while listener_running {
                match input_events.recv_timeout(Duration::from_millis(25)) {
                    Ok(event) => match state.handle(event) {
                        Some(InputAction::Sound(identifier)) if enabled.load(Ordering::Acquire) => {
                            if !matches!(identifier.as_str(), "left" | "right" | "middle_scroll")
                                || mouse_throttle.allow(std::time::Instant::now())
                            {
                                playback.play(&identifier)
                            }
                        }
                        Some(InputAction::ToggleEnabled) => {
                            let value = !enabled.load(Ordering::Acquire);
                            enabled.store(value, Ordering::Release);
                            playback.set_enabled(value);
                            let _ = thread_events.send(RuntimeEvent::ToggleApp(value));
                        }
                        _ => {}
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => listener_running = false,
                }
                for result in listener_status.try_iter() {
                    if let Err(message) = result {
                        let _ = thread_events.send(RuntimeEvent::Warning(message));
                        listener_running = false;
                    }
                }
                for message in audio_status.try_iter() {
                    let _ = thread_events.send(RuntimeEvent::Warning(message));
                }
            }
        })
        .unwrap_or_else(|error| {
            let _ = events.send(RuntimeEvent::Warning(format!(
                "Could not start input processing: {error}"
            )));
            std::thread::spawn(|| {})
        })
}

fn platform_notice() -> Option<String> {
    #[cfg(target_os = "macos")]
    if !macos_accessibility_client::accessibility::application_is_trusted() {
        return Some("Enable Accessibility for KeyJolt (or the terminal that launched it) in System Settings > Privacy & Security > Accessibility. Global sounds require this permission.".to_string());
    }
    #[cfg(target_os = "linux")]
    if std::env::var("XDG_SESSION_TYPE")
        .is_ok_and(|session| session.eq_ignore_ascii_case("wayland"))
    {
        return Some("Global sounds may not work in a Wayland session. Log in to an X11 session to use the global input listener.".to_string());
    }
    None
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

enum PreviewStateValue {
    Stopped,
    Playing,
    Finished,
    Error(SharedString),
}

impl PreviewStateValue {
    fn into_keyboard(self) -> view::keyboard_editor::PreviewState {
        match self {
            Self::Stopped => view::keyboard_editor::PreviewState::Stopped,
            Self::Playing => view::keyboard_editor::PreviewState::Playing,
            Self::Finished => view::keyboard_editor::PreviewState::Finished,
            Self::Error(message) => view::keyboard_editor::PreviewState::Error(message),
        }
    }

    fn into_mouse(self) -> view::mouse_editor::PreviewState {
        match self {
            Self::Stopped => view::mouse_editor::PreviewState::Stopped,
            Self::Playing => view::mouse_editor::PreviewState::Playing,
            Self::Finished => view::mouse_editor::PreviewState::Finished,
            Self::Error(message) => view::mouse_editor::PreviewState::Error(message),
        }
    }
}

enum AppPage {
    Home,
    KeyboardEditor(Entity<KeyboardEditorView>),
    MouseEditor(Entity<MouseEditorView>),
}

impl EventEmitter<AppShellEvent> for AppShell {}

impl EventEmitter<AppTitleBarEvent> for AppearancePicker {}

impl Focusable for AppearancePicker {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AppearancePicker {
    fn new(
        themes: Vec<ThemeDescriptor>,
        selection: AppearanceSelection,
        error_message: Option<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selection,
            themes,
            error_message,
        }
    }

    fn set_selection(&mut self, selection: AppearanceSelection, cx: &mut Context<Self>) {
        self.selection = selection;
        self.error_message = None;
        cx.notify();
    }
}

impl Render for AppearancePicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mode = self.selection.mode;
        let theme_rows = self
            .themes
            .iter()
            .filter(|theme| theme.mode == mode)
            .map(|theme| {
                let theme_id = theme.id.clone();
                let selected = theme_id == self.selection.theme_id;
                Button::new(SharedString::from(format!("appearance-theme-{theme_id}")))
                    .ghost()
                    .w_full()
                    .flex_shrink_0()
                    .justify_start()
                    .accessibility_label(theme.display_name.clone())
                    .selected(selected)
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .gap_2()
                            .child(IconName::Palette)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_left()
                                    .child(theme.display_name.clone()),
                            )
                            .child(
                                div()
                                    .w_4()
                                    .h_4()
                                    .flex_shrink_0()
                                    .when(selected, |indicator| indicator.child(IconName::Check)),
                            ),
                    )
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.emit(AppTitleBarEvent::ThemeSelected(theme_id.clone()));
                    }))
            })
            .collect::<Vec<_>>();

        v_flex()
            .w(px(280.))
            .gap_3()
            .track_focus(&self.focus_handle)
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child("Dark mode")
                    .child(div().flex_1())
                    .child(
                        Switch::new("appearance-dark-mode")
                            .checked(mode == AppearanceMode::Dark)
                            .accessibility_label("Dark mode")
                            .on_change(cx.listener(|this, dark, _, cx| {
                                if *dark != (this.selection.mode == AppearanceMode::Dark) {
                                    cx.emit(AppTitleBarEvent::ModeSelected(
                                        this.selection.mode.opposite(),
                                    ));
                                }
                            })),
                    ),
            )
            .child(
                v_flex().gap_1().child("Themes").child(
                    div()
                        .text_sm()
                        .child(format!("Current: {}", self.selection.theme_id)),
                ),
            )
            .child(
                div().w_full().h(px(360.)).flex_shrink_0().child(
                    v_flex()
                        .id("appearance-theme-list")
                        .size_full()
                        .gap_1()
                        .pr_3()
                        .children(theme_rows)
                        .overflow_y_scrollbar(),
                ),
            )
            .when_some(self.error_message.clone(), |panel, message| {
                panel.child(
                    div()
                        .id("appearance-error")
                        .w_full()
                        .max_h(px(100.))
                        .text_sm()
                        .text_color(cx.theme().danger)
                        .child(message)
                        .overflow_y_scrollbar(),
                )
            })
    }
}

impl AppTitleBar {
    fn new(
        themes: Vec<ThemeDescriptor>,
        selection: AppearanceSelection,
        error_message: Option<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        let appearance_picker =
            cx.new(|cx| AppearancePicker::new(themes, selection, error_message, cx));
        let picker_subscription =
            cx.subscribe(&appearance_picker, |_, _, event: &AppTitleBarEvent, cx| {
                cx.emit(event.clone());
            });

        Self {
            appearance_picker,
            popover_open: false,
            _picker_subscription: picker_subscription,
        }
    }

    fn set_selection(&mut self, selection: AppearanceSelection, cx: &mut Context<Self>) {
        self.appearance_picker.update(cx, |picker, cx| {
            picker.set_selection(selection, cx);
        });
        cx.notify();
    }

    fn set_error(&mut self, message: String, cx: &mut Context<Self>) {
        self.appearance_picker.update(cx, |picker, cx| {
            picker.error_message = Some(message);
            cx.notify();
        });
    }
}

impl Render for AppTitleBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let picker_focus = self.appearance_picker.focus_handle(cx);

        TitleBar::new()
            .child(div().flex().items_center().child("KeyJolt"))
            .child(div().flex_1())
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Popover::new("appearance-popover")
                            .anchor(Anchor::TopRight)
                            .offset(px(4.))
                            .overlay_closable(true)
                            .open(self.popover_open)
                            .track_focus(&picker_focus)
                            .trigger(
                                Button::new("settings")
                                    .ghost()
                                    .icon(IconName::Settings)
                                    .accessibility_label("Settings")
                                    .tooltip("Settings"),
                            )
                            .child(self.appearance_picker.clone())
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.popover_open = *open;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .child(
                                Button::new("github")
                                    .ghost()
                                    .icon(IconName::Github)
                                    .accessibility_label("Open KeyJolt on GitHub")
                                    .tooltip("GitHub repository")
                                    .on_click(|_, _, cx| {
                                        cx.open_url(GITHUB_REPOSITORY_URL);
                                    }),
                            ),
                    ),
            )
    }
}

impl AppShell {
    fn new(
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
        let title_bar =
            cx.new(|cx| AppTitleBar::new(themes.clone(), selection, appearance_error, cx));
        let title_bar_subscription = cx.subscribe_in(
            &title_bar,
            window,
            |this, _title_bar, event: &AppTitleBarEvent, window, cx| match event {
                AppTitleBarEvent::ModeSelected(mode) => {
                    let selection = pair_selection(
                        &AppearanceSelection {
                            mode: this.settings.appearance_mode,
                            theme_id: this.settings.appearance_theme_id.clone(),
                        },
                        *mode,
                        &this.themes,
                    )
                    .or_else(|| resolve_selection(*mode, None, &this.themes));
                    if let Some(selection) = selection {
                        this.apply_appearance(selection, window, cx);
                    }
                }
                AppTitleBarEvent::ThemeSelected(theme_id) => {
                    if let Some(theme) = this.themes.iter().find(|theme| &theme.id == theme_id) {
                        let selection = AppearanceSelection {
                            mode: theme.mode,
                            theme_id: theme.id.clone(),
                        };
                        this.apply_appearance(selection, window, cx);
                    } else {
                        this.title_bar.update(cx, |title_bar, cx| {
                            title_bar.set_error(format!("Theme not found: {theme_id}"), cx);
                        });
                    }
                }
            },
        );
        let settings_generation = Arc::new(AtomicU64::new(0));
        let (runtime_tx, runtime_rx) = mpsc::channel();
        let tray_icon = match tray::create(runtime_tx.clone()) {
            Ok(icon) => {
                tray_available.store(true, Ordering::Release);
                Some(icon)
            }
            Err(error) => {
                window.push_notification(
                    Notification::warning(format!("System tray unavailable: {error}"))
                        .title("System tray")
                        .placement(Anchor::BottomRight)
                        .autohide(true)
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.notify();
                            cx.hide();
                        })),
                        cx,
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
            window.push_notification(
                Notification::warning(message.clone())
                    .title("Preset or settings warning")
                    .placement(Anchor::BottomRight)
                    .autohide(true)
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.notify();
                        cx.hide();
                    })),
                cx,
            );
        }
        if let Some(message) = platform_notice() {
            window.push_notification(
                Notification::warning(message)
                    .title("Global input access")
                    .placement(Anchor::BottomRight)
                    .autohide(false)
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.notify();
                        cx.hide();
                    })),
                cx,
            );
        }

        let home_subscription = cx.subscribe_in(
            &home,
            window,
            |this, _home, event: &HomeEvent, window, cx| match event {
                HomeEvent::EditRequested(preset_id) => {
                    let preset_name = this.home.read(cx).active_preset_name();
                    let preset_data = this.home.read(cx).keyboard_preset_data(preset_id);

                    this.open_keyboard_editor(
                        Some(preset_id.clone()),
                        preset_name,
                        preset_data,
                        window,
                        cx,
                    );
                }

                HomeEvent::ConfigureMouseRequested(preset_id) => {
                    let preset_name = this.home.read(cx).active_mouse_preset_name();
                    let preset_data = this.home.read(cx).mouse_preset_data(preset_id);
                    this.open_mouse_editor(
                        Some(preset_id.clone()),
                        preset_name,
                        preset_data,
                        window,
                        cx,
                    );
                }

                HomeEvent::CreateKeyboardRequested => {
                    this.open_keyboard_editor(None, "".into(), None, window, cx);
                }

                HomeEvent::CreateMouseRequested => {
                    this.open_mouse_editor(None, "".into(), None, window, cx);
                }

                HomeEvent::ImportKeyboardRequested => {
                    this.start_import_preset(PresetKind::Keyboard, window, cx);
                }

                HomeEvent::ImportMousesRequested => {
                    this.start_import_preset(PresetKind::Mouse, window, cx);
                }

                HomeEvent::DeleteKeyboardRequested(id) => {
                    this.start_delete_preset(PresetKind::Keyboard, id.clone(), window, cx);
                }

                HomeEvent::DeleteMouseRequested(id) => {
                    this.start_delete_preset(PresetKind::Mouse, id.clone(), window, cx);
                }

                HomeEvent::SettingsChanged { settings, persist } => {
                    let keyboard_changed = this.settings.active_keyboard_preset_id
                        != settings.active_keyboard_preset_id;
                    let mouse_changed =
                        this.settings.active_mouse_preset_id != settings.active_mouse_preset_id;
                    let mut settings = settings.clone();
                    settings.appearance_mode = this.settings.appearance_mode;
                    settings.appearance_theme_id = this.settings.appearance_theme_id.clone();
                    this.settings = settings.clone();
                    this.playback.update_settings(&settings);
                    if keyboard_changed {
                        this.playback.reload(
                            PresetKind::Keyboard,
                            settings.active_keyboard_preset_id.as_deref().unwrap_or(""),
                        );
                    }
                    if mouse_changed {
                        this.playback.reload(
                            PresetKind::Mouse,
                            settings.active_mouse_preset_id.as_deref().unwrap_or(""),
                        );
                    }
                    cx.emit(AppShellEvent::SettingsChanged(settings.clone()));
                    if *persist {
                        let root = this.data_dir.clone();
                        let settings = settings.clone();
                        let generation =
                            this.settings_generation.fetch_add(1, Ordering::AcqRel) + 1;
                        let current_generation = this.settings_generation.clone();
                        this._settings_task = Some(cx.spawn_in(window, async move |this, cx| {
                            let result = cx
                                .background_spawn(async move {
                                    std::thread::sleep(Duration::from_millis(250));
                                    if current_generation.load(Ordering::Acquire) == generation {
                                        Some(settings_store::save_to(&root, &settings))
                                    } else {
                                        None
                                    }
                                })
                                .await;
                            if let Some(Err(error)) = result {
                                _ =
                                    this.update_in(cx, |_, window, cx| {
                                        window.push_notification(
                                            Notification::error(format!(
                                                "Could not save settings: {error}"
                                            ))
                                            .title("Settings save failed")
                                            .placement(Anchor::BottomRight)
                                            .autohide(true)
                                            .on_click(cx.listener(|_, _, _, cx| {
                                                cx.notify();
                                                cx.hide();
                                            })),
                                            cx,
                                        );
                                    });
                            }
                        }));
                    }
                }

                _ => {}
            },
        );

        let runtime_task = cx.spawn_in(window, async move |this, cx| {
            let mut receiver = runtime_rx;
            loop {
                let (next_receiver, result) = cx
                    .background_spawn(async move {
                        let result = receiver.recv_timeout(Duration::from_millis(250));
                        (receiver, result)
                    })
                    .await;
                receiver = next_receiver;
                match result {
                    Ok(RuntimeEvent::ToggleApp(enabled)) => {
                        _ = this.update_in(cx, |shell, window, cx| {
                            shell.home.update(cx, |home, cx| {
                                home.set_app_enabled_from_shortcut(enabled, cx)
                            });
                            window.push_notification(
                                Notification::info(if enabled {
                                    "Key and mouse sounds enabled."
                                } else {
                                    "Key and mouse sounds disabled."
                                })
                                .title("App status changed")
                                .placement(Anchor::BottomRight)
                                .autohide(true)
                                .on_click(cx.listener(
                                    |_, _, _, cx| {
                                        cx.notify();
                                        cx.hide();
                                    },
                                )),
                                cx,
                            );
                        });
                    }
                    Ok(RuntimeEvent::Warning(message)) => {
                        _ = this.update_in(cx, |_, window, cx| {
                            window.push_notification(
                                Notification::warning(message)
                                    .title("Global input listener")
                                    .placement(Anchor::BottomRight)
                                    .autohide(false)
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        cx.notify();
                                        cx.hide();
                                    })),
                                cx,
                            );
                        });
                    }
                    Ok(RuntimeEvent::TrayOpen) => {
                        _ = cx.update(|_, app| app.activate(true));
                        _ = this.update_in(cx, |_, window, _| {
                            #[cfg(target_os = "windows")]
                            set_windows_window_visibility(window, true);
                            window.activate_window();
                        });
                    }
                    Ok(RuntimeEvent::TrayExit) => {
                        _ = cx.update(|_, app| app.quit());
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

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
            themes,
            playback,
            _tray_icon: tray_icon,
            _runtime_thread: runtime_thread,
            _runtime_task: runtime_task,
        }
    }

    fn set_preview_state(
        &mut self,
        target: PreviewTarget,
        state: PreviewStateValue,
        cx: &mut Context<Self>,
    ) {
        match (&self.page, target) {
            (AppPage::KeyboardEditor(editor), PreviewTarget::Keyboard(file_id)) => {
                editor.update(cx, |editor, cx| {
                    editor.set_preview_state(file_id, state.into_keyboard(), cx);
                });
            }
            (AppPage::MouseEditor(editor), PreviewTarget::Mouse(file_id)) => {
                editor.update(cx, |editor, cx| {
                    editor.set_preview_state(file_id, state.into_mouse(), cx);
                });
            }
            _ => {}
        }
    }

    fn apply_appearance(
        &mut self,
        selection: AppearanceSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Err(message) = apply_theme(&selection, cx) {
            self.title_bar.update(cx, |title_bar, cx| {
                title_bar.set_error(message.clone(), cx);
            });
            window.push_notification(
                Notification::error(message)
                    .title("Theme change failed")
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
        self.settings.appearance_mode = selection.mode;
        self.settings.appearance_theme_id = selection.theme_id.clone();
        self.title_bar.update(cx, |title_bar, cx| {
            title_bar.set_selection(selection, cx);
        });
        self.persist_settings(window, cx);
    }

    fn persist_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let root = self.data_dir.clone();
        let settings = self.settings.clone();
        let generation = self.settings_generation.fetch_add(1, Ordering::AcqRel) + 1;
        let current_generation = self.settings_generation.clone();
        self._settings_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    std::thread::sleep(Duration::from_millis(250));
                    if current_generation.load(Ordering::Acquire) == generation {
                        Some(settings_store::save_to(&root, &settings))
                    } else {
                        None
                    }
                })
                .await;
            if let Some(Err(error)) = result {
                _ = this.update_in(cx, |_, window, cx| {
                    window.push_notification(
                        Notification::error(format!("Could not save settings: {error}"))
                            .title("Settings save failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                });
            }
        }));
    }

    fn start_import_preset(
        &mut self,
        kind: PresetKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.data_dir.clone();
        let window_handle = window.window_handle();
        let title = match kind {
            PresetKind::Keyboard => "Import keyboard preset",
            PresetKind::Mouse => "Import mouse preset",
        };

        cx.spawn(async move |this, cx| {
            let file = rfd::AsyncFileDialog::new()
                .set_title(title)
                .add_filter("KeyJolt preset", &["zip"])
                .pick_file()
                .await;
            let Some(file) = file else {
                return;
            };
            let source = file.path().to_path_buf();
            let result = cx
                .background_spawn(async move { storage::import_preset(&root, kind, &source) })
                .await;

            _ = window_handle.update(cx, |_, window, cx| {
                _ = this.update(cx, |shell, cx| match result {
                    Ok(preset) => {
                        let id: SharedString = preset.id.clone().into();
                        let name: SharedString = preset.name.clone().into();
                        let summary: SharedString = preset.summary.clone().into();
                        match kind {
                            PresetKind::Keyboard => {
                                if let Some(data) = preset.keyboard {
                                    shell.home.update(cx, |home, cx| {
                                        home.apply_saved_keyboard(
                                            id.clone(),
                                            name.clone(),
                                            summary.clone(),
                                            data,
                                            window,
                                            cx,
                                        );
                                    });
                                }
                            }
                            PresetKind::Mouse => {
                                if let Some(data) = preset.mouse {
                                    shell.home.update(cx, |home, cx| {
                                        home.apply_saved_mouse(
                                            id.clone(),
                                            name.clone(),
                                            summary.clone(),
                                            data,
                                            window,
                                            cx,
                                        );
                                    });
                                }
                            }
                        }
                        shell.playback.reload(kind, id.to_string());
                        cx.emit(AppShellEvent::PresetReloadRequested {
                            kind,
                            id: id.clone(),
                        });
                        window.push_notification(
                            Notification::success(format!("\"{name}\" was imported."))
                                .title("Preset imported")
                                .placement(Anchor::BottomRight)
                                .autohide(true)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.notify();
                                    cx.hide();
                                })),
                            cx,
                        );
                        cx.notify();
                    }
                    Err(error) => {
                        window.push_notification(
                            Notification::error(format!("Could not import preset: {error}"))
                                .title("Preset import failed")
                                .placement(Anchor::BottomRight)
                                .autohide(true)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.notify();
                                    cx.hide();
                                })),
                            cx,
                        );
                    }
                });
            });
        })
        .detach();
    }

    fn start_delete_preset(
        &mut self,
        kind: PresetKind,
        id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.data_dir.clone();
        let storage_id = id.clone();
        self._delete_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move { storage::delete_preset(&root, kind, &storage_id) })
                .await;
            _ = this.update_in(cx, move |shell, window, cx| match result {
                Ok(()) => {
                    shell.home.update(cx, |home, cx| {
                        home.remove_preset(kind, &id, window, cx);
                    });
                    window.push_notification(
                        Notification::success("The preset was deleted.")
                            .title("Preset deleted")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }
                Err(error) => {
                    window.push_notification(
                        Notification::error(format!("Could not delete preset: {error}"))
                            .title("Preset deletion failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }
            });
        }));
    }

    fn stop_preview(&mut self, target: Option<PreviewTarget>, cx: &mut Context<Self>) {
        let Some(active) = self.active_preview.as_ref() else {
            return;
        };
        if target.is_some_and(|target| target != active.target) {
            return;
        }

        let active = self.active_preview.take().unwrap();
        active.stop_requested.store(true, Ordering::Release);
        self.set_preview_state(active.target, PreviewStateValue::Stopped, cx);
    }

    fn start_preview(
        &mut self,
        target: PreviewTarget,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop_preview(None, cx);

        let id = self.next_preview_id;
        self.next_preview_id = self.next_preview_id.wrapping_add(1);
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker_stop_requested = stop_requested.clone();
        let volume = match target {
            PreviewTarget::Keyboard(_) => self.settings.effective_keyboard_volume(),
            PreviewTarget::Mouse(_) => self.settings.effective_mouse_volume(),
        };
        let task = cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    audio_preview::play_file(&path, worker_stop_requested, volume)
                })
                .await;

            _ = this.update_in(cx, |shell, window, cx| {
                shell.finish_preview(id, target, result, window, cx);
            });
        });

        self.active_preview = Some(ActivePreview {
            id,
            target,
            stop_requested,
            _task: task,
        });
        self.set_preview_state(target, PreviewStateValue::Playing, cx);
    }

    fn finish_preview(
        &mut self,
        id: u64,
        target: PreviewTarget,
        result: Result<audio_preview::PreviewEnd, String>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self
            .active_preview
            .as_ref()
            .is_some_and(|active| active.id == id && active.target == target)
        {
            return;
        }

        self.active_preview.take();
        let state = match result {
            Ok(audio_preview::PreviewEnd::Stopped) => PreviewStateValue::Stopped,
            Ok(audio_preview::PreviewEnd::Finished) => PreviewStateValue::Finished,
            Err(message) => PreviewStateValue::Error(message.into()),
        };
        self.set_preview_state(target, state, cx);
    }

    #[allow(clippy::too_many_arguments)]
    fn start_keyboard_save(
        &mut self,
        editor: Entity<KeyboardEditorView>,
        requested_id: Option<SharedString>,
        name: SharedString,
        summary: SharedString,
        data: KeyboardPresetData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing_user_preset = requested_id
            .as_deref()
            .is_some_and(|id| self.home.read(cx).is_user_keyboard_preset(id));
        let id: SharedString =
            storage::resolve_preset_id(requested_id.as_deref(), existing_user_preset).into();
        editor.update(cx, |editor, cx| {
            editor.set_save_status(view::keyboard_editor::SaveStatus::Saving, cx);
        });
        self.stop_preview(None, cx);
        let root = self.data_dir.clone();
        let worker_id = id.to_string();
        let worker_name = name.to_string();
        let worker_summary = summary.to_string();
        let worker_data = data.clone();
        self._save_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    storage::save_keyboard_preset(
                        &root,
                        &worker_id,
                        &worker_name,
                        &worker_summary,
                        &worker_data,
                    )?;
                    storage::load_preset(&root, PresetKind::Keyboard, &worker_id)
                })
                .await;
            _ = this.update_in(cx, move |shell, window, cx| match result {
                Ok(saved) => {
                    let saved_data = saved.keyboard.unwrap_or_else(|| data.clone());
                    shell.playback.reload(PresetKind::Keyboard, id.to_string());
                    shell.home.update(cx, |home, cx| {
                        home.apply_saved_keyboard(
                            id.clone(),
                            name.clone(),
                            summary.clone(),
                            saved_data,
                            window,
                            cx,
                        );
                    });
                    cx.emit(AppShellEvent::PresetReloadRequested {
                        kind: PresetKind::Keyboard,
                        id: id.clone(),
                    });
                    shell.editor_subscription.take();
                    shell.page = AppPage::Home;
                    window.push_notification(
                        Notification::success(format!("\"{name}\" was saved."))
                            .title("Keyboard preset saved")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                    cx.notify();
                }
                Err(error) => {
                    editor.update(cx, |editor, cx| {
                        editor.set_save_status(
                            view::keyboard_editor::SaveStatus::Failed(error.clone().into()),
                            cx,
                        );
                    });
                    window.push_notification(
                        Notification::error(format!("Could not save preset: {error}"))
                            .title("Keyboard preset save failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }
            });
        }));
    }

    #[allow(clippy::too_many_arguments)]
    fn start_mouse_save(
        &mut self,
        editor: Entity<MouseEditorView>,
        requested_id: Option<SharedString>,
        name: SharedString,
        summary: SharedString,
        data: MousePresetData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing_user_preset = requested_id
            .as_deref()
            .is_some_and(|id| self.home.read(cx).is_user_mouse_preset(id));
        let id: SharedString =
            storage::resolve_preset_id(requested_id.as_deref(), existing_user_preset).into();
        editor.update(cx, |editor, cx| {
            editor.set_save_status(view::mouse_editor::SaveStatus::Saving, cx);
        });
        self.stop_preview(None, cx);
        let root = self.data_dir.clone();
        let worker_id = id.to_string();
        let worker_name = name.to_string();
        let worker_summary = summary.to_string();
        let worker_data = data.clone();
        self._save_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    storage::save_mouse_preset(
                        &root,
                        &worker_id,
                        &worker_name,
                        &worker_summary,
                        &worker_data,
                    )?;
                    storage::load_preset(&root, PresetKind::Mouse, &worker_id)
                })
                .await;
            _ = this.update_in(cx, move |shell, window, cx| match result {
                Ok(saved) => {
                    let saved_data = saved.mouse.unwrap_or_else(|| data.clone());
                    shell.playback.reload(PresetKind::Mouse, id.to_string());
                    shell.home.update(cx, |home, cx| {
                        home.apply_saved_mouse(
                            id.clone(),
                            name.clone(),
                            summary.clone(),
                            saved_data,
                            window,
                            cx,
                        );
                    });
                    cx.emit(AppShellEvent::PresetReloadRequested {
                        kind: PresetKind::Mouse,
                        id: id.clone(),
                    });
                    shell.editor_subscription.take();
                    shell.page = AppPage::Home;
                    window.push_notification(
                        Notification::success(format!("\"{name}\" was saved."))
                            .title("Mouse preset saved")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                    cx.notify();
                }
                Err(error) => {
                    editor.update(cx, |editor, cx| {
                        editor.set_save_status(
                            view::mouse_editor::SaveStatus::Failed(error.clone().into()),
                            cx,
                        );
                    });
                    window.push_notification(
                        Notification::error(format!("Could not save preset: {error}"))
                            .title("Mouse preset save failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }
            });
        }));
    }

    fn open_keyboard_editor(
        &mut self,
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<KeyboardPresetData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop_preview(None, cx);
        let editor = KeyboardEditorView::view(preset_id, preset_name, preset_data, window, cx);

        let subscription = cx.subscribe_in(
            &editor,
            window,
            |this, editor_entity, event: &KeyboardEditorEvent, window, cx| match event {
                KeyboardEditorEvent::BackRequested => {
                    this.stop_preview(None, cx);
                    this.editor_subscription.take();
                    this.page = AppPage::Home;
                    cx.notify();
                }

                KeyboardEditorEvent::PreviewRequested { file_id, path } => {
                    this.start_preview(PreviewTarget::Keyboard(*file_id), path.clone(), window, cx);
                }

                KeyboardEditorEvent::StopPreviewRequested { file_id } => {
                    this.stop_preview(file_id.map(PreviewTarget::Keyboard), cx);
                }

                KeyboardEditorEvent::PlaybackStateChanged {
                    state: view::keyboard_editor::PreviewState::Error(message),
                    ..
                } => {
                    window.push_notification(
                        Notification::error(message.clone())
                            .title("Audio preview failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }

                KeyboardEditorEvent::PlaybackStateChanged { .. } => {}

                KeyboardEditorEvent::SaveRequested {
                    preset_id,
                    preset_name,
                    summary,
                    data,
                } => {
                    this.start_keyboard_save(
                        editor_entity.clone(),
                        preset_id.clone(),
                        preset_name.clone(),
                        summary.clone(),
                        data.clone(),
                        window,
                        cx,
                    );
                }
            },
        );

        self.editor_subscription = Some(subscription);
        self.page = AppPage::KeyboardEditor(editor);

        cx.notify();
    }

    fn open_mouse_editor(
        &mut self,
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<MousePresetData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop_preview(None, cx);
        let editor = MouseEditorView::view(preset_id, preset_name, preset_data, window, cx);
        let subscription = cx.subscribe_in(
            &editor,
            window,
            |this, editor_entity, event: &MouseEditorEvent, window, cx| match event {
                MouseEditorEvent::BackRequested => {
                    this.stop_preview(None, cx);
                    this.editor_subscription.take();
                    this.page = AppPage::Home;
                    cx.notify();
                }

                MouseEditorEvent::PreviewRequested { file_id, path } => {
                    this.start_preview(PreviewTarget::Mouse(*file_id), path.clone(), window, cx);
                }

                MouseEditorEvent::StopPreviewRequested { file_id } => {
                    this.stop_preview(file_id.map(PreviewTarget::Mouse), cx);
                }

                MouseEditorEvent::PlaybackStateChanged {
                    state: view::mouse_editor::PreviewState::Error(message),
                    ..
                } => {
                    window.push_notification(
                        Notification::error(message.clone())
                            .title("Audio preview failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }

                MouseEditorEvent::PlaybackStateChanged { .. } => {}

                MouseEditorEvent::SaveRequested {
                    preset_id,
                    preset_name,
                    summary,
                    data,
                } => {
                    this.start_mouse_save(
                        editor_entity.clone(),
                        preset_id.clone(),
                        preset_name.clone(),
                        summary.clone(),
                        data.clone(),
                        window,
                        cx,
                    );
                }
            },
        );

        self.editor_subscription = Some(subscription);
        self.page = AppPage::MouseEditor(editor);
        cx.notify();
    }
}

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

fn load_theme_catalog(cx: &mut App) -> (Vec<ThemeDescriptor>, Vec<String>) {
    let mut themes = Vec::new();
    let mut errors = Vec::new();
    for (index, content) in appearance::bundled_theme_sets().enumerate() {
        let descriptors = match appearance::descriptors_from_theme_sets([content]) {
            Ok(descriptors) => descriptors,
            Err(error) => {
                errors.push(format!("Bundled theme set {}: {error}", index + 1));
                continue;
            }
        };
        if let Err(error) = ThemeRegistry::global_mut(cx).load_themes_from_str(content) {
            let family = descriptors
                .first()
                .map(|theme| theme.family.as_str())
                .unwrap_or("Unnamed theme set");
            errors.push(format!("Could not load {family}: {error}"));
            continue;
        }
        for descriptor in descriptors {
            if ThemeRegistry::global(cx)
                .themes()
                .contains_key(descriptor.id.as_str())
            {
                themes.push(descriptor);
            } else {
                errors.push(format!("Theme was not registered: {}", descriptor.id));
            }
        }
    }

    for (mode, config) in ThemeRegistry::global(cx).default_themes() {
        let mode = match mode {
            ThemeMode::Light => AppearanceMode::Light,
            ThemeMode::Dark => AppearanceMode::Dark,
        };
        themes.push(ThemeDescriptor {
            id: config.name.to_string(),
            display_name: config.name.to_string(),
            mode,
            family: "Default".to_string(),
            is_default: true,
        });
    }
    themes.sort_by(|left, right| {
        left.mode.cmp(&right.mode).then(
            left.display_name
                .to_lowercase()
                .cmp(&right.display_name.to_lowercase()),
        )
    });
    themes.dedup_by(|right, left| right.id == left.id);
    (themes, errors)
}

fn apply_theme(selection: &AppearanceSelection, cx: &mut App) -> Result<(), String> {
    let config = ThemeRegistry::global(cx)
        .themes()
        .get(selection.theme_id.as_str())
        .cloned()
        .ok_or_else(|| format!("Theme not found: {}", selection.theme_id))?;

    Theme::update(cx, |theme| theme.apply_config(&config));
    Ok(())
}

fn main() {
    gpui_kit::application()
        .with_assets(AppAssets)
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx: &mut App| {
            gpui_kit::init(cx);
            let (themes, mut appearance_errors) = load_theme_catalog(cx);
            let (_, startup_settings, _) = settings_store::load();
            let startup_selection = resolve_selection(
                startup_settings.appearance_mode,
                Some(&startup_settings.appearance_theme_id),
                &themes,
            )
            .or_else(|| resolve_selection(startup_settings.appearance_mode, None, &themes))
            .unwrap_or(AppearanceSelection {
                mode: AppearanceMode::Light,
                theme_id: appearance::default_theme_id(AppearanceMode::Light).to_string(),
            });
            if let Err(error) = apply_theme(&startup_selection, cx) {
                appearance_errors.push(error);
            }
            let appearance_error = if appearance_errors.is_empty() {
                None
            } else {
                Some(appearance_errors.join("\n"))
            };
            let tray_available = Arc::new(AtomicBool::new(false));
            let close_tray_available = tray_available.clone();

            let bounds = Bounds::centered(None, size(px(620.), px(900.)), cx);

            let locked_size = size(px(620.0), px(900.0));

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(locked_size),
                    is_resizable: false,
                    icon: None,
                    ..TitleBar::window_options()
                },
                move |window, cx| {
                    window.on_window_should_close(cx, move |_window, _cx| {
                        if close_tray_available.load(Ordering::Acquire) {
                            #[cfg(target_os = "windows")]
                            set_windows_window_visibility(_window, false);
                            #[cfg(not(target_os = "windows"))]
                            _cx.hide();
                            false
                        } else {
                            true
                        }
                    });
                    let home = HomeView::view(window, cx);
                    let app = cx.new(|cx| {
                        AppShell::new(
                            home,
                            tray_available,
                            themes.clone(),
                            appearance_error.clone(),
                            window,
                            cx,
                        )
                    });

                    cx.new(|cx| Root::new(app, window, cx))
                },
            )
            .expect("open window");

            cx.activate(true);
        });
}

#[cfg(target_os = "windows")]
fn set_windows_window_visibility(window: &Window, visible: bool) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW, ShowWindow};

    let Ok(handle) = <Window as HasWindowHandle>::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let command = if visible { SW_SHOW } else { SW_HIDE };
    // SAFETY: GPUI supplies a live HWND for this window, and this only changes its visibility on the UI thread.
    unsafe {
        ShowWindow(handle.hwnd.get() as _, command);
    }
}
