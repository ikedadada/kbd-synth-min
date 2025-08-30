use crate::synth::{
    adsr::Adsr,
    note::Note,
    osc::{Osc, Waveform},
};

#[derive(Clone, Copy, Default)]
struct Voice {
    on: bool,
    note: Note,
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
    pub fn new(sr: f32, waveform: Waveform) -> Self {
        Self {
            sr,
            voices: [Default::default(); MAX_VOICES],
            master_volume: 0.2,
            attack: 0.0,
            decay: 0.5,
            sustain: 1.0,
            release: 0.5,
            waveform,
        }
    }

    pub fn note_on(&mut self, note: Note) {
        let mut adsr = Adsr::new(self.attack, self.decay, self.sustain, self.release, self.sr);
        adsr.note_on();
        if let Some(v) = self.voices.iter_mut().find(|v| v.on && v.note == note) {
            v.asdr = adsr;
            return;
        }
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.on) {
            voice.on = true;
            voice.note = note;
            voice.phase = 0.0;
            voice.asdr = adsr;
            voice.osc = Osc::new(note.into(), self.sr, self.waveform);
        }
    }

    pub fn note_off(&mut self, note: Note) {
        for v in self.voices.iter_mut() {
            if v.on && v.note == note {
                v.asdr.note_off();
            }
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

    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_volume = vol.clamp(0.0, 1.0);
    }

    pub fn set_adsr(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.attack = a.max(0.0);
        self.decay = d.max(0.0);
        self.sustain = s.clamp(0.0, 1.0);
        self.release = r.max(0.0);
        // 進行中のボイスのASDRを更新（次のサンプルから反映）
        for v in self.voices.iter_mut() {
            if v.on {
                v.asdr = Adsr::new(self.attack, self.decay, self.sustain, self.release, self.sr);
                if v.asdr.is_active() {
                    v.asdr.note_on();
                }
            }
        }
    }

    pub fn set_waveform(&mut self, wf: Waveform) {
        self.waveform = wf;
        for v in self.voices.iter_mut() {
            v.osc.set_waveform(wf);
        }
    }
}
