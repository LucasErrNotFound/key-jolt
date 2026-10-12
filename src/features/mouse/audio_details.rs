use super::MouseEditorView;
pub(super) use crate::features::preset_editor::AudioDetails;
use crate::features::preset_editor::analyze_file;
use gpui_kit::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct AnalysisJob {
    pub(super) id: u64,
    pub(super) _task: Task<()>,
    pub(super) cancelled: Arc<AtomicBool>,
}

impl Drop for AnalysisJob {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl MouseEditorView {
    pub(super) fn refresh_audio_details(&mut self, cx: &mut Context<Self>) {
        for file in &self.files {
            self.audio_details
                .entry(file.id)
                .or_insert(AudioDetails::Pending);
        }
        if self.analysis_job.is_some() {
            return;
        }
        let Some(file) = self.files.iter().find(|file| {
            matches!(
                self.audio_details.get(&file.id),
                Some(AudioDetails::Pending)
            )
        }) else {
            return;
        };
        let id = file.id;
        let path = file.path.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        let task = cx.spawn(async move |this, cx| {
            let details = cx
                .background_spawn(async move { analyze_file(&path, &worker_cancelled) })
                .await;
            _ = this.update(cx, |view, cx| {
                if view.analysis_job.as_ref().is_none_or(|job| job.id != id) {
                    return;
                }
                if view.files.iter().any(|file| file.id == id) {
                    view.audio_details.insert(id, details);
                }
                view.analysis_job = None;
                view.refresh_audio_details(cx);
                cx.notify();
            });
        });
        self.analysis_job = Some(AnalysisJob {
            id,
            _task: task,
            cancelled,
        });
    }
}
