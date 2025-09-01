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
pub(crate) mod web_entry {
    use crate::gui::EguiUi;
    use crate::synth::{Msg, SharedBus, Synth, Waveform};
    use eframe::{App, WebOptions, WebRunner};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    use std::cell::RefCell;
    use wasm_bindgen::closure::Closure;

    thread_local! {
        static AUDIO_CONTEXT: RefCell<Option<web_sys::AudioContext>> = const { RefCell::new(None) };
        static WORKLET_NODE: RefCell<Option<web_sys::AudioWorkletNode>> = const { RefCell::new(None) };
    }

    async fn init_audio(bus: SharedBus) -> Result<(), JsValue> {
        use web_sys::js_sys::{Array, ArrayBuffer, Float32Array, Object, Reflect};

        // Prefer interactive/low-latency context if available
        let ctx = if true {
            let opts = web_sys::AudioContextOptions::new();
            // Set latencyHint: 'interactive'
            opts.set_latency_hint(&JsValue::from_str("interactive"));
            web_sys::AudioContext::new_with_context_options(&opts)?
        } else {
            web_sys::AudioContext::new()?
        };
        let sr = ctx.sample_rate();

        // AudioWorklet-only path
        // Early return if AudioWorklet is unavailable
        let aw = match ctx.audio_worklet() {
            Ok(aw) => aw,
            Err(_) => return Ok(()),
        };

        let p = aw.add_module("worklet/synth-processor.js")?;
        wasm_bindgen_futures::JsFuture::from(p).await?;

        let opts = web_sys::AudioWorkletNodeOptions::new();
        opts.set_channel_count(2);
        let counts = Array::new();
        counts.push(&JsValue::from_f64(2.0));
        opts.set_output_channel_count(&counts);

        let node = web_sys::AudioWorkletNode::new_with_options(&ctx, "synth-processor", &opts)?;
        let port = node.port()?;
        port.start();

        // Prepare synth and message handler
        let mut synth = Synth::new(sr, Waveform::Sine, None);
        let bus_for_cb = bus.clone();

        let port_for_cb = port.clone();
        // Pre-fill one contiguous buffer of target blocks to reduce startup glitch
        {
            let total = 128 * 8; // target blocks (keep in sync with worklet)
            let arr = Float32Array::new_with_length(total as u32);
            // Fill directly into typed array to avoid an extra copy
            for i in 0..total {
                let v = synth.next_sample().clamp(-1.0, 1.0);
                arr.set_index(i as u32, v);
            }
            let buf: ArrayBuffer = arr.buffer();
            let payload = Object::new();
            let _ = Reflect::set(&payload, &JsValue::from_str("mono"), &arr);
            let _ = port.post_message_with_transferable(&payload, &Array::of1(&buf));
        }

        let onmsg = Closure::wrap(Box::new(move |ev: web_sys::MessageEvent| {
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

            let data = ev.data();
            let need_val = Reflect::get(&data, &JsValue::from_str("need")).ok();
            let need_frames = need_val.and_then(|v| v.as_f64()).unwrap_or(128.0) as usize;
            // Round up to quantum multiple
            let quantum = 128usize;
            let total = need_frames.div_ceil(quantum) * quantum;

            let arr = Float32Array::new_with_length(total as u32);
            for i in 0..total {
                let v = synth.next_sample().clamp(-1.0, 1.0);
                arr.set_index(i as u32, v);
            }
            let buf: ArrayBuffer = arr.buffer();
            let payload = Object::new();
            let _ = Reflect::set(&payload, &JsValue::from_str("mono"), &arr);
            let _ = port_for_cb.post_message_with_transferable(&payload, &Array::of1(&buf));
        }) as Box<dyn FnMut(_)>);
        port.set_onmessage(Some(onmsg.as_ref().unchecked_ref()));
        onmsg.forget();

        node.connect_with_audio_node(&ctx.destination())?;
        AUDIO_CONTEXT.with(|c| c.borrow_mut().replace(ctx));
        WORKLET_NODE.with(|n| n.borrow_mut().replace(node));

        // Resume on user gesture
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
        resume.forget();

        Ok(())
    }

    // Allow resuming AudioContext from elsewhere in the crate (e.g. on first key press)
    pub fn try_resume_audio() {
        AUDIO_CONTEXT.with(|c| {
            if let Some(ctx) = c.borrow().as_ref() {
                let _ = ctx.resume();
            }
        });
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
        init_audio(bus.clone()).await?;

        let options = WebOptions::default();
        let runner = WebRunner::new();
        let bus_for_ui = bus.clone();
        runner
            .start(
                canvas,
                options,
                Box::new(
                    move |_cc| -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
                        Ok(Box::new(EguiUi::new(bus_for_ui.clone())))
                    },
                ),
            )
            .await
    }
}
