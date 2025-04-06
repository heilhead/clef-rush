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
