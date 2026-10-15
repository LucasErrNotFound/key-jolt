use super::HomeStartup;
use crate::settings::store::AppSettings;

impl HomeStartup {
    pub(crate) fn load() -> Self {
        let (data_dir, mut settings, settings_warning) = crate::settings::store::load();
        let (loaded_presets, preset_warnings) = crate::presets::load_presets(&data_dir);
        settings.active_keyboard_preset_id = crate::presets::startup_preset_id(
            crate::presets::PresetKind::Keyboard,
            settings.active_keyboard_preset_id.as_deref(),
            &loaded_presets,
        );
        settings.active_mouse_preset_id = crate::presets::startup_preset_id(
            crate::presets::PresetKind::Mouse,
            settings.active_mouse_preset_id.as_deref(),
            &loaded_presets,
        );
        let mut startup_warnings = preset_warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect::<Vec<_>>();
        if let Some(warning) = settings_warning {
            startup_warnings.push(format!(
                "Settings could not be loaded. Defaults are active: {warning}"
            ));
        }
        Self {
            data_dir,
            settings,
            loaded_presets,
            startup_warnings,
        }
    }

    pub(crate) fn add_startup_warning(&mut self, warning: String) {
        self.startup_warnings.push(warning);
    }

    pub(crate) fn settings(&self) -> &AppSettings {
        &self.settings
    }
}
