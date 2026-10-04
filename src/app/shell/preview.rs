use super::{ActivePreview, AppPage, AppShell, PreviewState, PreviewTarget};

use gpui_kit::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

impl AppShell {
    pub(super) fn set_preview_state(
        &mut self,
        target: PreviewTarget,
        state: PreviewState,
        cx: &mut Context<Self>,
    ) {
        match (&self.page, target) {
            (AppPage::KeyboardEditor(editor), PreviewTarget::Keyboard(file_id)) => {
                editor.update(cx, |editor, cx| {
                    editor.set_preview_state(file_id, state, cx);
                });
            }
            (AppPage::MouseEditor(editor), PreviewTarget::Mouse(file_id)) => {
                editor.update(cx, |editor, cx| {
                    editor.set_preview_state(file_id, state, cx);
                });
            }
            _ => {}
        }
    }

    pub(super) fn stop_preview(&mut self, target: Option<PreviewTarget>, cx: &mut Context<Self>) {
        let Some(active) = self.active_preview.as_ref() else {
            return;
        };
        if target.is_some_and(|target| target != active.target) {
            return;
        }

        let active = self.active_preview.take().unwrap();
        active.stop_requested.store(true, Ordering::Release);
        self.set_preview_state(active.target, PreviewState::Stopped, cx);
    }

    pub(super) fn start_preview(
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
                    crate::audio::preview::play_file(&path, worker_stop_requested, volume)
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
        self.set_preview_state(target, PreviewState::Playing, cx);
    }

    pub(super) fn finish_preview(
        &mut self,
        id: u64,
        target: PreviewTarget,
        result: Result<crate::audio::preview::PreviewEnd, String>,
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
            Ok(crate::audio::preview::PreviewEnd::Stopped) => PreviewState::Stopped,
            Ok(crate::audio::preview::PreviewEnd::Finished) => PreviewState::Finished,
            Err(message) => PreviewState::Error(message.into()),
        };
        self.set_preview_state(target, state, cx);
    }
}
