use {app::App, tap::TapFallible, wasm_bindgen::prelude::*};

pub mod app;
pub mod input;
pub mod util;
pub mod verovio;

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    tracing_wasm::set_as_global_default();

    // modal, loading_spinners, layout, checkbox

    input::refresh_ports();

    let _ = iced::application("Piano Trainer", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run_with(App::boot)
        .tap_err(|err| {
            tracing::error!(?err, "iced app failed");
        });
}
