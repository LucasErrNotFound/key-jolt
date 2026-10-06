use super::HomeStartup;
use crate::settings::store::AppSettings;

impl HomeStartup {
    pub(crate) fn load() -> Self {
        let (data_dir, settings, settings_warning) = crate::settings::store::load();
        let (loaded_presets, preset_warnings) = crate::presets::load_presets(&data_dir);
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
