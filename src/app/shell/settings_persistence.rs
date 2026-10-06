use crate::platform::startup;
use crate::settings::store::{AppSettings, save_to};
use std::path::Path;

pub(super) struct SettingsWriter {
    pub(super) run_at_startup: bool,
}

impl SettingsWriter {
    pub(super) fn save(&mut self, root: &Path, settings: &AppSettings) -> Result<(), String> {
        persist_change(
            self.run_at_startup,
            settings.run_at_startup,
            startup::set_enabled,
            || save_to(root, settings),
        )?;
        self.run_at_startup = settings.run_at_startup;
        Ok(())
    }
}

fn persist_change(
    previous: bool,
    requested: bool,
    mut configure: impl FnMut(bool) -> Result<(), String>,
    save: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if previous == requested {
        return save();
    }
    configure(requested)?;
    if let Err(error) = save() {
        return match configure(previous) {
            Ok(()) => Err(error),
            Err(rollback) => Err(format!(
                "{error}. Could not restore the previous startup registration: {rollback}"
            )),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn failed_registration_does_not_save_the_requested_setting() {
        let saved = Cell::new(false);
        let result = persist_change(
            false,
            true,
            |_| Err("denied".to_string()),
            || {
                saved.set(true);
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!saved.get());
    }

    #[test]
    fn failed_save_restores_the_previous_registration() {
        let mut requests = Vec::new();
        let result = persist_change(
            true,
            false,
            |value| {
                requests.push(value);
                Ok(())
            },
            || Err("disk full".to_string()),
        );
        assert!(result.is_err());
        assert_eq!(requests, [false, true]);
    }

    #[test]
    fn successful_change_registers_before_saving() {
        let registered = Cell::new(false);
        let result = persist_change(
            false,
            true,
            |value| {
                registered.set(value);
                Ok(())
            },
            || {
                assert!(registered.get());
                Ok(())
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn unrelated_settings_do_not_touch_startup_registration() {
        assert!(
            persist_change(
                true,
                true,
                |_| panic!("startup must stay untouched"),
                || Ok(())
            )
            .is_ok()
        );
    }

    #[test]
    fn rollback_failure_is_reported() {
        let mut calls = 0;
        let error = persist_change(
            false,
            true,
            |_| {
                calls += 1;
                if calls == 1 {
                    Ok(())
                } else {
                    Err("rollback denied".to_string())
                }
            },
            || Err("disk full".to_string()),
        )
        .unwrap_err();
        assert!(error.contains("disk full"));
        assert!(error.contains("rollback denied"));
    }
}
