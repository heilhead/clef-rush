use {wasm_bindgen::prelude::*, wasm_bindgen_futures::js_sys::JsString};

#[wasm_bindgen]
extern "C" {
    type Verovio;

    #[wasm_bindgen(static_method_of = Verovio)]
    async fn init();

    #[wasm_bindgen(static_method_of = Verovio)]
    async fn ping();

    #[wasm_bindgen(static_method_of = Verovio, js_name = getOptions)]
    async fn get_options() -> JsString;

    #[wasm_bindgen(static_method_of = Verovio, js_name = setOptions)]
    async fn set_options(opts: JsString);

    #[wasm_bindgen(static_method_of = Verovio, js_name = resetOptions)]
    async fn reset_options();

    #[wasm_bindgen(static_method_of = Verovio, js_name = convertToSVG)]
    async fn convert_to_svg(mei: JsString) -> JsString;
}

pub async fn initialize() {
    tracing::info!("initializing verovio...");
    Verovio::init().await;
    Verovio::ping().await;
    Verovio::set_options(include_str!("../resources/verovio_options.json").into()).await;
    tracing::info!("ready");
}

pub async fn convert_to_svg(mei: String) -> String {
    Verovio::convert_to_svg(mei.into()).await.into()
}

pub async fn update_svg() -> String {
    tracing::info!("generating svg...");
    let instant = instant::Instant::now();
    let result: String = Verovio::convert_to_svg(include_str!("../resources/landmarks.mei").into())
        .await
        .into();
    let elapsed = instant.elapsed();
    tracing::info!(?elapsed, "svg");

    result
}
