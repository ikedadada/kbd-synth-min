use crate::libs::{
    adsr::Adsr,
    osc::{Osc, Waveform},
};

#[derive(Clone, Copy, Default)]
struct Voice {
    on: bool,
    note: f32,
    phase: f32,
    asdr: Adsr,
    osc: Osc,
}

const MAX_VOICES: usize = 16;

pub struct Synth {
    sr: f32,
    voices: [Voice; MAX_VOICES],
    master_volume: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    waveform: Waveform,
}

impl Synth {
    pub fn new(sr: f32) -> Self {
        Self {
            sr,
            voices: [Default::default(); MAX_VOICES],
            master_volume: 0.2,
            attack: 0.0,
            decay: 0.5,
            sustain: 0.0,
            release: 0.0,
            waveform: Waveform::Sine,
        }
    }

    pub fn note_on(&mut self, note: f32) {
        let mut adsr = Adsr::new(self.attack, self.decay, self.sustain, self.release, self.sr);
        adsr.note_on();
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.on) {
            voice.on = true;
            voice.note = note;
            voice.phase = 0.0;
            voice.asdr = adsr;
            voice.osc = Osc::new(note, self.sr, self.waveform);
        }
    }

    pub fn note_off(&mut self, note: f32) {
        println!("Note off: {}", note);
        if let Some(voice) = self
            .voices
            .iter_mut()
            .find(|v| v.on && (v.note - note).abs() < f32::EPSILON)
        {
            voice.on = false;
            voice.asdr.note_off();
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.master_volume == 0.0 {
            return 0.0;
        }
        let mut sample = 0.0;
        for voice in self.voices.iter_mut() {
            if voice.on {
                let env = voice.asdr.next_sample();
                if env <= 0.0 {
                    voice.on = false;
                    continue;
                }
                let osc_sample = voice.osc.next_sample();
                sample += osc_sample * env;
            }
        }
        sample * self.master_volume
    }
}
