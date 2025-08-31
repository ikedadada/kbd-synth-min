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
    use crate::synth::{Msg, SharedBus, Synth, Waveform};
    use crate::gui::EguiUi;
    use eframe::{App, WebOptions, WebRunner};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    use std::cell::RefCell;
    use wasm_bindgen::closure::Closure;

    thread_local! {
        static AUDIO_CONTEXT: RefCell<Option<web_sys::AudioContext>> = RefCell::new(None);
    }

    fn init_audio(bus: SharedBus) -> Result<(), JsValue> {
        let ctx = web_sys::AudioContext::new()?;
        let sr = ctx.sample_rate();

        // Use ScriptProcessorNode for simplicity (works broadly; low-latency enough here)
        let buffer_size: u32 = 1024; // power of two
        let channels_out: u32 = 2;
        let proc = ctx
            .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
                buffer_size,
                0,
                channels_out,
            )?;

        let mut synth = Synth::new(sr, Waveform::Sine, None);
        let bus_for_cb = bus.clone();
        let onaudio = Closure::wrap(Box::new(move |e: web_sys::AudioProcessingEvent| {
            // Drain bus messages
            while let Some(msg) = bus_for_cb.q.pop() {
                match msg {
                    Msg::NoteOn { note } => synth.note_on(note),
                    Msg::NoteOff { note } => synth.note_off(note),
                    Msg::SetMasterVolume(v) => synth.set_master_volume(v),
                    Msg::SetAdsr { a, d, s, r } => synth.set_adsr(a, d, s, r),
                    Msg::SetWaveform(wf) => synth.set_waveform(wf),
                    Msg::SetFilter(ft) => synth.set_filter(ft),
                }
            }

            let output = match e.output_buffer() {
                Ok(buf) => buf,
                Err(_) => return,
            };
            let frames = output.length() as usize;
            let num_ch = output.number_of_channels() as usize;

            // Generate mono then copy to all output channels
            let mut mono = vec![0.0f32; frames];
            for s in mono.iter_mut() {
                *s = synth.next_sample();
            }

            for ch in 0..num_ch {
                if let Ok(arr) = output.get_channel_data(ch as u32) {
                    arr.copy_from(&mono);
                }
            }
        }) as Box<dyn FnMut(_)>);

        proc.set_onaudioprocess(Some(onaudio.as_ref().unchecked_ref()));
        onaudio.forget(); // keep callback alive

        // Connect to destination to start processing
        proc.connect_with_audio_node(&ctx.destination())?;

        // Store context so it stays alive
        AUDIO_CONTEXT.with(|c| c.borrow_mut().replace(ctx));

        // Try to resume on first user gesture (keydown / pointerdown)
        let resume = Closure::wrap(Box::new(move || {
            AUDIO_CONTEXT.with(|c| {
                if let Some(ctx) = c.borrow().as_ref() {
                    let _ = ctx.resume();
                }
            });
        }) as Box<dyn FnMut()>);
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let et: &web_sys::EventTarget = window.as_ref();
        et.add_event_listener_with_callback("keydown", resume.as_ref().unchecked_ref())?;
        et.add_event_listener_with_callback("pointerdown", resume.as_ref().unchecked_ref())?;
        resume.forget(); // keep listeners alive

        Ok(())
    }

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

        // Shared bus between UI and audio
        let bus = SharedBus::default();
        init_audio(bus.clone())?;

        let options = WebOptions::default();
        let runner = WebRunner::new();
        let bus_for_ui = bus.clone();
        runner
            .start(
                canvas,
                options,
                Box::new(move |_cc| -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(Box::new(EguiUi::new(bus_for_ui.clone())))
                }),
            )
            .await
    }
}
