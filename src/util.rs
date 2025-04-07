pub async fn sleep(delay: i32) {
    let mut cb = |resolve: js_sys::Function, reject: js_sys::Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay)
            .unwrap();
    };

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut cb))
        .await
        .unwrap();
}
