// Force windows to not spawn a terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod export;
mod state;
mod view;

use app::App;

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Fix: Force XWayland usage until iced/wgpu fixes the problem
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("WAYLAND_DISPLAY", "");
    }

    #[allow(unused_mut)]
    let mut app = iced::application(App::new, App::update, App::view)
        .antialiasing(true)
        .title(App::title)
        .theme(App::theme)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES);

    #[cfg(target_arch = "wasm32")]
    {
        use iced::Font;

        app = app
            .font(include_bytes!("../fonts/FiraSans-Regular.ttf"))
            .default_font(Font::with_name("Fira Sans"));
    }

    app.run()
}
