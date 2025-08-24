use std::f32::consts::PI;

use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

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
    let config = StreamConfig::from(supported);

    println!("Using device: {}", device.name().unwrap());
    println!("Sample format: {:?}", sample_format);
    println!("Stream config: {:?}", config);

    // 音のパラメータ
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let phase_size = 2.0 * PI;
    let freq_h = 440.0_f32;
    let phase_inc = phase_size * freq_h / sample_rate; // A4 = 440Hz

    // ストリーム作成
    let err_fn = |err: cpal::StreamError| eprintln!("an error occurred on stream: {}", err);
    let stream = match sample_format {
        SampleFormat::F32 => {
            build_stream::<f32>(&device, &config, channels, phase_size, phase_inc, err_fn)
                .expect("Failed to build stream")
        }
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
    phase_size: f32,
    phase_inc: f32,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut phase = 0.0_f32; // 位相の時間軸: [0.0, phase_size)
    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let volume = 0.2; // 0.0 ~ 1.0
            for frame in data.chunks_mut(channels) {
                let wave = phase.sin(); // 波の形
                let value: f32 = wave * volume; // 音量を考慮した波形
                phase += phase_inc; // 位相を更新
                // 位相が phase_size を超えたら0に戻す
                if phase > phase_size {
                    phase = 0.0;
                }

                let s: T = cpal::Sample::from_sample(value);

                if channels == 2 {
                    // ステレオ
                    frame[0] = s; // 左
                    frame[1] = s; // 右
                    continue;
                } else {
                    // ステレオ以外ならモノラルとして扱う
                    frame[0] = s; // モノラル
                }
            }
        },
        err_fn,
        None,
    )
}
