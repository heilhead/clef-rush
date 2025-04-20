use {wasm_bindgen::JsValue, wasm_bindgen_futures::JsFuture};

pub async fn sleep(delay: i32) {
    let mut cb = |resolve: js_sys::Function, _: js_sys::Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay)
            .unwrap();
    };

    JsFuture::from(js_sys::Promise::new(&mut cb)).await.unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No global `window` object")]
    NoWindow,

    #[error("No `document` object")]
    NoDocument,

    #[error("No `body` object")]
    NoBody,

    #[error("Failed to enter fullscreen: {0}")]
    Fullscreen(String),
}

pub fn toggle_fullscreen() -> Result<(), Error> {
    let document = web_sys::window()
        .ok_or(Error::NoWindow)?
        .document()
        .ok_or(Error::NoDocument)?;

    if document.fullscreen_element().is_some() {
        document.exit_fullscreen();
        Ok(())
    } else {
        document
            .body()
            .ok_or(Error::NoBody)?
            .request_fullscreen()
            .map_err(|err| Error::Fullscreen(js_error_to_string(err)))
    }
}

fn js_error_to_string(err: JsValue) -> String {
    err.as_string().unwrap_or_else(|| "<no data>".to_owned())
}
