use gpui_kit::*;

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
        if let Some(bytes) = SelectedIcons.load(path)? {
            return Ok(Some(bytes));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.extend(SelectedIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
