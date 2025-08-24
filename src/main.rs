use std::sync::mpsc::Receiver;

use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::libs::{
    adsr::Adsr,
    osc::{Osc, Waveform},
};

mod libs;

#[derive(Debug, Clone, Copy)]
enum Control {
    NoteOn,
    NoteOff,
}

fn main() {
    let mut args = std::env::args();
    let w = args.nth(1).unwrap_or_else(|| "sine".into());
    let waveform = match w.as_str() {
        "sine" => Waveform::Sine,
        "square" => Waveform::Square,
        "sawtooth" => Waveform::Sawtooth,
        "triangle" => Waveform::Triangle,
        _ => {
            eprintln!("Unknown waveform: {}, using sine", w);
            Waveform::Sine
        }
    };
    let master_volume = 0.2_f32;

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
    config.buffer_size = cpal::BufferSize::Fixed(8192);

    println!("Using device: {}", device.name().unwrap());
    println!("Sample format: {:?}", sample_format);
    println!("Stream config: {:?}", config);

    // 音のパラメータ
    let sample_rate = config.sample_rate.0 as f32;
    let freq_h = 440.0_f32;

    let osc = Osc::new(freq_h, sample_rate, waveform);
    let adsr = Adsr::new(0.0, 1.0, 0.0, 0.0, sample_rate);

    let channels = config.channels as usize;

    // コントロール用チャネル
    let (tx, rx) = std::sync::mpsc::channel::<Control>();

    std::thread::spawn(move || {
        // コントロール信号を送信
        tx.send(Control::NoteOn).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        tx.send(Control::NoteOff).unwrap();
    });

    // ストリーム作成
    let err_fn = |err: cpal::StreamError| eprintln!("an error occurred on stream: {}", err);
    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(BuildStreamParams {
            device: &device,
            config: &config,
            channels,
            master_volume,
            osc,
            adsr,
            rx,
            err_fn: Box::new(err_fn),
        })
        .expect("Failed to build stream"),
        _ => unimplemented!("Only f32 sample format is implemented"),
    };

    stream.play().expect("Failed to play stream");
    std::thread::sleep(std::time::Duration::from_secs(2));
    stream.pause().expect("Failed to pause stream");
}

struct BuildStreamParams<'a> {
    device: &'a cpal::Device,
    config: &'a StreamConfig,
    channels: usize,
    master_volume: f32,
    osc: Osc,
    adsr: Adsr,
    rx: Receiver<Control>,
    err_fn: Box<dyn Fn(cpal::StreamError) + Send + 'static>,
}

fn build_stream<T>(params: BuildStreamParams) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
{
    let mut osc = params.osc;
    let mut adsr = params.adsr;

    params.device.build_output_stream(
        params.config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(control) = params.rx.try_recv() {
                match control {
                    Control::NoteOn => adsr.note_on(),
                    Control::NoteOff => adsr.note_off(),
                }
            }

            for frames in data.chunks_mut(params.channels) {
                let sample = if params.master_volume == 0.0 {
                    0.0
                } else {
                    osc.next_sample() * adsr.next_sample() * params.master_volume + 1e-20f32
                };
                let s: T = cpal::Sample::from_sample(sample);
                for frame in frames {
                    *frame = s;
                }
            }
        },
        params.err_fn,
        None,
    )
}
