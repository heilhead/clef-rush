use std::borrow::Cow;

pub async fn sleep(delay: i32) {
    let mut cb = |resolve: js_sys::Function, _: js_sys::Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay)
            .unwrap();
    };

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut cb))
        .await
        .unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum FullscreenError {
    #[error("No global `window` object")]
    NoWindow,

    #[error("No `document` object")]
    NoDocument,

    #[error("No `body` object")]
    NoBody,

    #[error("Failed to enter fullscreen: {0}")]
    FailedToEnter(String),
}

pub fn toggle_fullscreen() -> Result<(), FullscreenError> {
    let window = web_sys::window().ok_or(FullscreenError::NoWindow)?;
    let document = window.document().ok_or(FullscreenError::NoDocument)?;
    let body = document.body().ok_or(FullscreenError::NoBody)?;

    if document.fullscreen_element().is_none() {
        body.request_fullscreen().map_err(|err| {
            FullscreenError::FailedToEnter(
                err.as_string().unwrap_or_else(|| "<no data>".to_owned()),
            )
        })
    } else {
        document.exit_fullscreen();
        Ok(())
    }
}

pub struct DropMonitor {
    name: Cow<'static, str>,
}

impl DropMonitor {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        tracing::info!("monitor created: {}", &name);
        Self { name }
    }
}

impl Drop for DropMonitor {
    fn drop(&mut self) {
        tracing::info!("monitor dropped: {}", &self.name);
    }
}
