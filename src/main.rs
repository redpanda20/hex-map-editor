// Force windows to not spawn a terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod export;
mod state;
mod view;

use app::App;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Fix: Force XWayland usage until iced/wgpu fixes the problem
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("WAYLAND_DISPLAY", "");
    }

    iced::application(App::new, App::update, App::view)
        .antialiasing(true)
        .title(App::title)
        .theme(App::theme)
        .run()

    // iced::application(App::title, App::update, App::view)
    //     .theme(|_| iced::Theme::Dark)
    //     .antialiasing(true)
    //     .run_with(App::new)
}

// WASM entry point
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    main().expect("iced application failed");
}
