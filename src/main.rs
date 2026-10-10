#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod activity;
mod app;
mod app_assets;
mod audio;
mod features;
mod input;
mod persistence;
mod platform;
mod presets;
mod runtime;
mod settings;

fn main() {
    app::run();
}
