use gpui_kit::*;

pub(crate) const APP_ICON_PATH: &str = "icons/key-jolt.svg";
pub(crate) const APP_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/key-jolt.png");
const APP_LOGO_BYTES: &[u8] = include_bytes!("../assets/icons/key-jolt.svg");

gpui_kit::assets::icon_assets!(
    SelectedIcons,
    [
        Keyboard,
        Mouse,
        Import,
        Pencil,
        Wrench,
        VolumeX,
        Volume2,
        Settings,
        Github,
        Upload,
        CircleCheck,
        Play,
        FileVolume,
        CircleX,
        Close,
        Check,
        ArrowLeft
    ]
);

#[derive(Clone, Copy, Debug, Default)]
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path == APP_ICON_PATH {
            return Ok(Some(std::borrow::Cow::Borrowed(APP_LOGO_BYTES)));
        }
        if let Some(bytes) = SelectedIcons.load(path)? {
            return Ok(Some(bytes));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.extend(SelectedIcons.list(path)?);
        if APP_ICON_PATH.starts_with(path) {
            paths.push(APP_ICON_PATH.into());
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
