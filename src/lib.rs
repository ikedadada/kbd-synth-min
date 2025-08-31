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
    use eframe::WebOptions;
    use wasm_bindgen::prelude::*;

    // Better error messages in the browser console on panic
    #[wasm_bindgen(start)]
    pub async fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();

        let options = WebOptions::default();
        eframe::start_web(
            "the_canvas_id",
            options,
            Box::new(|_cc| Box::new(EguiUi::new(SharedBus::default()))),
        )
        .await
    }
}
