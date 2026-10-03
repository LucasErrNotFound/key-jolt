use std::sync::mpsc::Sender;

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use super::RuntimeEvent;

pub fn create(sender: Sender<RuntimeEvent>) -> Result<TrayIcon, String> {
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
    let icon = Icon::from_rgba(icon_pixels(), 32, 32).map_err(|error| error.to_string())?;
    TrayIconBuilder::new()
        .with_tooltip("KeyJolt")
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_menu_on_right_click(true)
        .with_icon(icon)
        .build()
        .map_err(|error| error.to_string())
}

fn icon_pixels() -> Vec<u8> {
    let mut pixels = vec![0; 32 * 32 * 4];
    for y in 5..27 {
        for x in 5..27 {
            let is_key = y < 19 || ((13..19).contains(&x) && y >= 19);
            if is_key {
                let offset = (y * 32 + x) * 4;
                let color = if (8..13).contains(&y) && (9..23).contains(&x) {
                    [80, 220, 190, 255]
                } else {
                    [38, 75, 110, 255]
                };
                pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }
    pixels
}
