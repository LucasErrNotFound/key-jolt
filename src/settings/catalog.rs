use super::appearance::{self, AppearanceMode, AppearanceSelection, ThemeDescriptor};
use gpui_kit::App;
use gpui_kit::component::theme::{Theme, ThemeMode, ThemeRegistry};

pub(crate) fn load_theme_catalog(cx: &mut App) -> (Vec<ThemeDescriptor>, Vec<String>) {
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
            display_name: config.name.to_string().into(),
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

pub(crate) fn apply_theme(selection: &AppearanceSelection, cx: &mut App) -> Result<(), String> {
    let config = ThemeRegistry::global(cx)
        .themes()
        .get(selection.theme_id.as_str())
        .cloned()
        .ok_or_else(|| format!("Theme not found: {}", selection.theme_id))?;

    Theme::update(cx, |theme| theme.apply_config(&config));
    Ok(())
}
