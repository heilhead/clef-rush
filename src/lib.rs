#![no_main]

use {app::App, tap::TapFallible, wasm_bindgen::prelude::*};

pub mod app;
pub mod input;
pub mod keyboard;
pub mod piano;
pub mod util;
pub mod verovio;

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    tracing_wasm::set_as_global_default();

    // Initialize midi input as early as possible.
    let _ = input::port_list();

    let _ = iced::application(App::boot, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(app::Font::Title.source())
        .default_font(app::Font::default().into())
        .run()
        .tap_err(|err| {
            tracing::error!(?err, "iced app failed");
        });
}
