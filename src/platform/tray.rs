use std::sync::mpsc::Sender;

use gpui_kit::{App, Image, ImageFormat};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app_assets::APP_ICON_BYTES;
use crate::runtime::RuntimeEvent;

pub fn create(sender: Sender<RuntimeEvent>, cx: &App) -> Result<TrayIcon, String> {
    let image = Image::from_bytes(ImageFormat::Png, APP_ICON_BYTES.to_vec())
        .to_image_data(cx.svg_renderer())
        .map_err(|error| format!("Could not decode the app icon: {error}"))?;
    let size = image.size(0);
    let width =
        u32::try_from(size.width.0).map_err(|_| "The app icon has an invalid width".to_string())?;
    let height = u32::try_from(size.height.0)
        .map_err(|_| "The app icon has an invalid height".to_string())?;
    let mut pixels = image
        .as_bytes(0)
        .ok_or_else(|| "The app icon has no image frame".to_string())?
        .to_vec();
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let icon = Icon::from_rgba(pixels, width, height).map_err(|error| error.to_string())?;
    let open = MenuItem::with_id("open-keyjolt", "Open KeyJolt", true, None);
    let exit = MenuItem::with_id("exit-keyjolt", "Exit", true, None);
    let menu = Menu::with_items(&[&open, &exit]).map_err(|error| error.to_string())?;
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let action = match event.id.as_ref() {
            "open-keyjolt" => Some(RuntimeEvent::TrayOpen),
            "exit-keyjolt" => Some(RuntimeEvent::TrayExit),
            _ => None,
        };
        if let Some(action) = action {
            let _ = sender.send(action);
        }
    }));
    TrayIconBuilder::new()
        .with_tooltip("KeyJolt")
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_menu_on_right_click(true)
        .with_icon(icon)
        .build()
        .map_err(|error| error.to_string())
}
