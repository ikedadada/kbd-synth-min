use std::{io::stdout, time::Duration};

use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossterm::{
    event::{
        self, Event, KeyCode, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::libs::{
    osc::Waveform,
    shared_bus::{Msg, SharedBus},
    synth::Synth,
};

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
    let mut config = StreamConfig::from(supported);
    config.buffer_size = cpal::BufferSize::Fixed(64);

    println!("Using device: {}", device.name().unwrap());
    println!("Sample format: {:?}", sample_format);
    println!("Stream config: {:?}", config);

    // 音のパラメータ
    let sample_rate = config.sample_rate.0 as f32;

    let channels = config.channels as usize;

    // ストリーム作成
    let err_fn = |err: cpal::StreamError| eprintln!("an error occurred on stream: {}", err);
    let bus = SharedBus::new();
    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(BuildStreamParams {
            device: &device,
            config: &config,
            bus: bus.clone(),
            channels,
            sample_rate,
            waveform,
            err_fn: Box::new(err_fn),
        })
        .expect("Failed to build stream"),
        _ => unimplemented!("Only f32 sample format is implemented"),
    };

    stream.play().expect("Failed to play stream");
    wait_for_event(bus).expect("Failed to wait for event");

    stream.pause().expect("Failed to pause stream");
}

fn wait_for_event(bus: SharedBus) -> Result<(), std::io::Error> {
    enable_raw_mode()?;

    // ★ これをループ前に入れる
    execute!(
        stdout(),
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES          // ← Press/Release/Repeat を有効化
            | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
    )?;

    loop {
        if !event::poll(Duration::from_millis(2))? {
            continue;
        }
        if let Event::Key(key_event) = event::read()? {
            let note = match key_event.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Char('z') => 261.626,
                KeyCode::Char('x') => 293.665,
                KeyCode::Char('c') => 329.628,
                KeyCode::Char('v') => 349.228,
                KeyCode::Char('b') => 392.0,
                KeyCode::Char('n') => 440.0,
                KeyCode::Char('m') => 493.883,
                KeyCode::Char(',') => 523.251,
                _ => continue,
            };
            let _ = match key_event.kind {
                event::KeyEventKind::Press => bus.q.push(Msg::NoteOn { note }),
                event::KeyEventKind::Release => bus.q.push(Msg::NoteOff { note }),
                event::KeyEventKind::Repeat => Ok(()),
            };
        }
    }
    // ★ 終了時に元に戻す
    execute!(stdout(), PopKeyboardEnhancementFlags)?;
    disable_raw_mode()?;
    Ok(())
}

struct BuildStreamParams<'a> {
    device: &'a cpal::Device,
    config: &'a StreamConfig,
    bus: SharedBus,
    channels: usize,
    sample_rate: f32,
    waveform: Waveform,
    err_fn: Box<dyn Fn(cpal::StreamError) + Send + 'static>,
}

fn build_stream<T>(params: BuildStreamParams) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
{
    let mut synth = Synth::new(params.sample_rate, params.waveform);
    let bus = params.bus;

    params.device.build_output_stream(
        params.config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Some(msg) = bus.q.pop() {
                match msg {
                    Msg::NoteOn { note } => {
                        synth.note_on(note);
                    }
                    Msg::NoteOff { note } => {
                        synth.note_off(note);
                    }
                }
            }
            for frames in data.chunks_mut(params.channels) {
                let sample = synth.next_sample();
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
