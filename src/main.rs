// Force windows to not spawn a terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
pub mod domain;
pub mod infrastructure;
pub mod theme;
pub mod ui;

use app::App;

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Fix: Force XWayland usage until iced/wgpu fixes the problem
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("WAYLAND_DISPLAY", "");
    }

    let app = iced::application(App::boot, App::update, App::view)
        .antialiasing(true)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .font(iced_fonts::LUCIDE_FONT_BYTES)
        .font(include_bytes!("../fonts/IBMPlexSans-Variable.ttf"))
        .default_font(iced::Font::with_name("IBM Plex Sans"));

    app.run()
}
