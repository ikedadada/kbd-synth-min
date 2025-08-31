pub mod synth {
    mod adsr;
    mod engine;
    mod filter;
    mod note;
    mod osc;
    mod shared_bus;
    // Re-export primary types to avoid deep paths
    pub use engine::Synth;
    pub use filter::FilterType;
    pub use note::Note;
    pub use osc::Waveform;
    pub use shared_bus::Msg;
    pub use shared_bus::SharedBus;
}

pub mod gui {
    mod app;
    pub use app::EguiUi;
}

// WASM entry point for web build
#[cfg(target_arch = "wasm32")]
mod web_entry {
    use crate::{gui::EguiUi, synth::SharedBus};
    use eframe::{App, WebOptions, WebRunner};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    // Better error messages in the browser console on panic
    #[wasm_bindgen(start)]
    pub async fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .ok_or_else(|| JsValue::from_str("canvas not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("failed to cast to HtmlCanvasElement"))?;

        let options = WebOptions::default();
        let runner = WebRunner::new();
        runner
            .start(
                canvas,
                options,
                Box::new(|_cc| -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(Box::new(EguiUi::new(SharedBus::default())))
                }),
            )
            .await
    }
}
