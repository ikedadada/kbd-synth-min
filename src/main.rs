#![cfg(not(target_arch = "wasm32"))]
use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use eframe::{NativeOptions, egui};
use kbd_synth_min::{
    audio::core::render_block,
    gui::EguiUi,
    synth::{FilterType, SharedBus, Synth, Waveform},
};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // デバイスと出力設定
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    // 既定の出力設定
    let supported = device.default_output_config().expect("No supported config");
    let sample_format = supported.sample_format();
    // Stream 用に変換
    let mut config = StreamConfig::from(supported);
    config.buffer_size = cpal::BufferSize::Fixed(64);

    println!("Using device: {}", device.name().unwrap());
    println!("Sample format: {:?}", sample_format);
    println!("Stream config: {:?}", config);

    // 音のパラメータ
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let bus = SharedBus::default();

    // ストリーム作成
    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(BuildStreamParams {
            device: &device,
            config: &config,
            bus: bus.clone(),
            channels,
            sample_rate,
            waveform: Waveform::Sine,
            filter: None,
            err_fn: Box::new(|err| eprintln!("an error occurred on stream: {}", err)),
        })
        .expect("Failed to build stream"),
        _ => unimplemented!("Only f32 sample format is implemented"),
    };

    stream.play().expect("Failed to play stream");

    let mut options = NativeOptions::default();
    options.viewport.inner_size = Some(egui::vec2(480.0, 320.0));
    let result = eframe::run_native(
        "Kbd Synth",
        options,
        Box::new(|_cc| Ok(Box::new(EguiUi::new(bus)))),
    );
    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}

struct BuildStreamParams<'a> {
    device: &'a cpal::Device,
    config: &'a StreamConfig,
    bus: SharedBus,
    channels: usize,
    sample_rate: f32,
    waveform: Waveform,
    filter: Option<FilterType>,
    err_fn: Box<dyn Fn(cpal::StreamError) + Send + 'static>,
}

fn build_stream<T>(params: BuildStreamParams) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
{
    let mut synth = Synth::new(params.sample_rate, params.waveform, params.filter);
    let bus = params.bus;
    let channels = params.channels;
    let mut scratch: Vec<f32> = Vec::new();

    params.device.build_output_stream(
        params.config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let nframes = data.len() / channels;
            if scratch.len() != nframes {
                scratch.resize(nframes, 0.0);
            }
            render_block(&mut synth, &bus, &mut scratch);
            for (i, frame) in data.chunks_mut(channels).enumerate() {
                let s: T = cpal::Sample::from_sample(scratch[i]);
                for ch in frame {
                    *ch = s;
                }
            }
        },
        params.err_fn,
        None,
    )
}
