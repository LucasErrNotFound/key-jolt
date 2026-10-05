use super::shell::AppShell;
use crate::app_assets::AppAssets;
use crate::features::home::{HomeStartup, HomeView};
#[cfg(target_os = "windows")]
use crate::platform::window::set_windows_window_visibility;
use crate::settings::appearance;
use crate::settings::appearance::{AppearanceMode, AppearanceSelection, resolve_selection};

use crate::settings::catalog::{apply_theme, load_theme_catalog};

use gpui_kit::component::TitleBar;

use gpui_kit::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) fn run() {
    gpui_kit::application()
        .with_assets(AppAssets)
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx: &mut App| {
            gpui_kit::init(cx);
            cx.spawn(async move |cx| {
                let startup = cx.background_spawn(async { HomeStartup::load() }).await;
                cx.update(move |cx| {
                    let (themes, mut appearance_errors) = load_theme_catalog(cx);
                    let startup_settings = startup.settings();
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

                    gpui_kit::open_window(
                        WindowOptions {
                            window_bounds: Some(WindowBounds::Windowed(bounds)),
                            window_min_size: Some(locked_size),
                            is_resizable: false,
                            icon: None,
                            ..TitleBar::window_options()
                        },
                        cx,
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
                            let home = HomeView::view(startup, window, cx);
                            cx.new(|cx| {
                                AppShell::new(
                                    home,
                                    tray_available,
                                    themes.clone(),
                                    appearance_error.clone(),
                                    window,
                                    cx,
                                )
                            })
                        },
                    )
                    .expect("open window");

                    cx.activate(true);
                });
            })
            .detach();
        });
}
