use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::libs::osc::{Osc, Waveform};

mod libs;

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

    // デバイスと出力設定
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    // 既定の出力設定
    let supported = device.default_output_config().expect("No supported config");
    let sample_format = supported.sample_format();
    // Stream 用に変換
    let config = StreamConfig::from(supported);

    println!("Using device: {}", device.name().unwrap());
    println!("Sample format: {:?}", sample_format);
    println!("Stream config: {:?}", config);

    // 音のパラメータ
    let sample_rate = config.sample_rate.0 as f32;
    let freq_h = 440.0_f32;

    let osc = Osc::new(freq_h, sample_rate, waveform);

    let channels = config.channels as usize;

    // ストリーム作成
    let err_fn = |err: cpal::StreamError| eprintln!("an error occurred on stream: {}", err);
    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(&device, &config, channels, osc, err_fn)
            .expect("Failed to build stream"),
        _ => unimplemented!("Only f32 sample format is implemented"),
    };

    stream.play().expect("Failed to play stream");
    std::thread::sleep(std::time::Duration::from_secs(2));
    stream.pause().expect("Failed to pause stream");
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    osc: Osc,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
{
    let mut block: Vec<f32> = Vec::new();
    let mut osc = osc;
    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // ブロック合成
            let frames = data.len() / channels;
            block.resize(frames, 0.0);

            for sample in block.iter_mut() {
                *sample = osc.next_sample();
            }

            match channels {
                1 => {
                    // モノラル
                    for (frame, &sample) in data.chunks_mut(channels).zip(block.iter()) {
                        let s: T = cpal::Sample::from_sample(sample);
                        frame[0] = s;
                    }
                }
                2 => {
                    // ステレオ
                    for (frame, &sample) in data.chunks_mut(channels).zip(block.iter()) {
                        let s: T = cpal::Sample::from_sample(sample);
                        frame[0] = s; // 左
                        frame[1] = s; // 右
                    }
                }
                _ => {
                    // その他のチャンネル数はモノラルとして扱う
                    for (frames, &sample) in data.chunks_mut(channels).zip(block.iter()) {
                        let s: T = cpal::Sample::from_sample(sample);
                        for frame in frames {
                            *frame = s;
                        }
                    }
                }
            }
        },
        err_fn,
        None,
    )
}
