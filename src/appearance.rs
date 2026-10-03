use serde::{Deserialize, Serialize};

pub const DEFAULT_LIGHT_THEME_ID: &str = "Default Light";
pub const DEFAULT_DARK_THEME_ID: &str = "Default Dark";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppearanceMode {
    #[default]
    Light,
    Dark,
}

impl AppearanceMode {
    pub fn opposite(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

pub fn default_theme_id(mode: AppearanceMode) -> &'static str {
    match mode {
        AppearanceMode::Light => DEFAULT_LIGHT_THEME_ID,
        AppearanceMode::Dark => DEFAULT_DARK_THEME_ID,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeDescriptor {
    pub id: String,
    pub display_name: String,
    pub mode: AppearanceMode,
    pub family: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppearanceSelection {
    pub mode: AppearanceMode,
    pub theme_id: String,
}

#[derive(Deserialize)]
struct ThemeSetMetadata {
    name: String,
    themes: Vec<ThemeMetadata>,
}

#[derive(Deserialize)]
struct ThemeMetadata {
    name: String,
    mode: AppearanceMode,
    #[serde(default)]
    is_default: bool,
}

pub fn bundled_theme_sets() -> impl Iterator<Item = &'static str> {
    [
        include_str!("../assets/themes/adventure.json"),
        include_str!("../assets/themes/alduin.json"),
        include_str!("../assets/themes/asciinema.json"),
        include_str!("../assets/themes/aurora.json"),
        include_str!("../assets/themes/ayu.json"),
        include_str!("../assets/themes/catppuccin.json"),
        include_str!("../assets/themes/everforest.json"),
        include_str!("../assets/themes/fahrenheit.json"),
        include_str!("../assets/themes/flexoki.json"),
        include_str!("../assets/themes/gruvbox.json"),
        include_str!("../assets/themes/harper.json"),
        include_str!("../assets/themes/hybrid.json"),
        include_str!("../assets/themes/jellybeans.json"),
        include_str!("../assets/themes/kibble.json"),
        include_str!("../assets/themes/macos-classic.json"),
        include_str!("../assets/themes/mellifluous.json"),
        include_str!("../assets/themes/molokai.json"),
        include_str!("../assets/themes/solarized.json"),
        include_str!("../assets/themes/spaceduck.json"),
        include_str!("../assets/themes/tokyonight.json"),
        include_str!("../assets/themes/twilight.json"),
    ]
    .into_iter()
}

pub fn descriptors_from_theme_sets<'a>(
    sets: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<ThemeDescriptor>, String> {
    sets.into_iter()
        .map(|content| {
            let set: ThemeSetMetadata =
                serde_json::from_str(content).map_err(|error| error.to_string())?;
            Ok(set
                .themes
                .into_iter()
                .map(|theme| ThemeDescriptor {
                    id: theme.name.clone(),
                    display_name: theme.name,
                    mode: theme.mode,
                    family: set.name.clone(),
                    is_default: theme.is_default,
                })
                .collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|groups| groups.into_iter().flatten().collect())
}

pub fn resolve_selection(
    mode: AppearanceMode,
    theme_id: Option<&str>,
    themes: &[ThemeDescriptor],
) -> Option<AppearanceSelection> {
    let theme = theme_id
        .and_then(|id| {
            themes
                .iter()
                .find(|theme| theme.id == id && theme.mode == mode)
        })
        .or_else(|| default_theme(mode, themes))?;

    Some(AppearanceSelection {
        mode,
        theme_id: theme.id.clone(),
    })
}

pub fn pair_selection(
    selection: &AppearanceSelection,
    target_mode: AppearanceMode,
    themes: &[ThemeDescriptor],
) -> Option<AppearanceSelection> {
    let current = themes
        .iter()
        .find(|theme| theme.id == selection.theme_id && theme.mode == selection.mode)
        .or_else(|| default_theme(selection.mode, themes))?;
    let target = themes
        .iter()
        .find(|theme| theme.mode == target_mode && theme.family == current.family)
        .or_else(|| default_theme(target_mode, themes))?;

    Some(AppearanceSelection {
        mode: target_mode,
        theme_id: target.id.clone(),
    })
}

fn default_theme(mode: AppearanceMode, themes: &[ThemeDescriptor]) -> Option<&ThemeDescriptor> {
    themes
        .iter()
        .find(|theme| theme.mode == mode && theme.is_default)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(id: &str, mode: AppearanceMode, family: &str, is_default: bool) -> ThemeDescriptor {
        ThemeDescriptor {
            id: id.to_string(),
            display_name: id.to_string(),
            mode,
            family: family.to_string(),
            is_default,
        }
    }

    fn themes() -> Vec<ThemeDescriptor> {
        vec![
            theme("Default Light", AppearanceMode::Light, "Default", true),
            theme("Default Dark", AppearanceMode::Dark, "Default", true),
            theme("Latte", AppearanceMode::Light, "Catppuccin", false),
            theme("Mocha", AppearanceMode::Dark, "Catppuccin", false),
            theme("One Light", AppearanceMode::Light, "One", false),
        ]
    }

    #[test]
    fn mode_change_selects_family_counterpart() {
        let selection = AppearanceSelection {
            mode: AppearanceMode::Light,
            theme_id: "Latte".to_string(),
        };

        let paired = pair_selection(&selection, AppearanceMode::Dark, &themes()).unwrap();

        assert_eq!(paired.theme_id, "Mocha");
    }

    #[test]
    fn mode_change_uses_default_when_family_has_no_counterpart() {
        let selection = AppearanceSelection {
            mode: AppearanceMode::Light,
            theme_id: "One Light".to_string(),
        };

        let paired = pair_selection(&selection, AppearanceMode::Dark, &themes()).unwrap();

        assert_eq!(paired.theme_id, DEFAULT_DARK_THEME_ID);
    }

    #[test]
    fn family_pairing_round_trips_in_both_directions() {
        let light = AppearanceSelection {
            mode: AppearanceMode::Light,
            theme_id: "Latte".to_string(),
        };
        let dark = pair_selection(&light, AppearanceMode::Dark, &themes()).unwrap();
        let returned = pair_selection(&dark, AppearanceMode::Light, &themes()).unwrap();

        assert_eq!(returned, light);

        let dark = AppearanceSelection {
            mode: AppearanceMode::Dark,
            theme_id: "Mocha".to_string(),
        };
        let light = pair_selection(&dark, AppearanceMode::Light, &themes()).unwrap();
        let returned = pair_selection(&light, AppearanceMode::Dark, &themes()).unwrap();

        assert_eq!(returned, dark);
    }

    #[test]
    fn missing_or_unknown_theme_id_uses_mode_default() {
        let themes = themes();

        let missing = resolve_selection(AppearanceMode::Light, None, &themes).unwrap();
        let unknown =
            resolve_selection(AppearanceMode::Dark, Some("Removed theme"), &themes).unwrap();

        assert_eq!(missing.theme_id, DEFAULT_LIGHT_THEME_ID);
        assert_eq!(unknown.theme_id, DEFAULT_DARK_THEME_ID);
    }
}
