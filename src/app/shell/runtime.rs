use super::AppShell;
#[cfg(target_os = "windows")]
use crate::platform::window::set_windows_window_visibility;
use crate::runtime::RuntimeEvent;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use std::sync::mpsc;
use std::time::Duration;

impl AppShell {
    pub(super) fn start_runtime_events(
        receiver: std::sync::mpsc::Receiver<RuntimeEvent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<()> {
        cx.spawn_in(window, async move |this, cx| {
            let mut receiver = receiver;
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
        })
    }
}
