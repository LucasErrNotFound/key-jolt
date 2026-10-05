use gpui_kit::Window;
use rfd::AsyncFileDialog;
use std::sync::atomic::{AtomicBool, Ordering};

static FILE_DIALOG_OPEN: AtomicBool = AtomicBool::new(false);

pub(crate) struct FileDialogGuard<'a> {
    active: &'a AtomicBool,
}

impl<'a> FileDialogGuard<'a> {
    fn try_acquire(active: &'a AtomicBool) -> Option<Self> {
        active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| Self { active })
    }
}

impl Drop for FileDialogGuard<'_> {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

pub(crate) fn begin_file_dialog(
    window: &Window,
) -> Option<(FileDialogGuard<'static>, AsyncFileDialog)> {
    let guard = FileDialogGuard::try_acquire(&FILE_DIALOG_OPEN)?;
    let dialog = AsyncFileDialog::new().set_parent(window);
    Some((guard, dialog))
}

#[cfg(test)]
mod tests {
    use super::FileDialogGuard;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn rejects_another_dialog_until_the_first_closes() {
        let active = AtomicBool::new(false);
        let first = FileDialogGuard::try_acquire(&active).unwrap();

        assert!(FileDialogGuard::try_acquire(&active).is_none());
        assert!(active.load(Ordering::Acquire));

        drop(first);

        assert!(!active.load(Ordering::Acquire));
        assert!(FileDialogGuard::try_acquire(&active).is_some());
    }

    #[test]
    fn releases_the_dialog_on_an_early_return() {
        fn cancel_dialog(active: &AtomicBool) -> Option<()> {
            let _guard = FileDialogGuard::try_acquire(active)?;
            None
        }

        let active = AtomicBool::new(false);

        assert_eq!(cancel_dialog(&active), None);
        assert!(FileDialogGuard::try_acquire(&active).is_some());
    }

    #[test]
    fn concurrent_requests_have_only_one_owner() {
        use std::sync::Barrier;

        let active = AtomicBool::new(false);
        let owners = std::sync::atomic::AtomicUsize::new(0);
        let start = Barrier::new(8);
        let acquired = Barrier::new(8);

        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| {
                    start.wait();
                    let guard = FileDialogGuard::try_acquire(&active);
                    if guard.is_some() {
                        owners.fetch_add(1, Ordering::Relaxed);
                    }
                    acquired.wait();
                    drop(guard);
                });
            }
        });

        assert_eq!(owners.load(Ordering::Relaxed), 1);
        assert!(FileDialogGuard::try_acquire(&active).is_some());
    }
}
