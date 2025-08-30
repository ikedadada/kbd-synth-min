use crate::synth::{
    adsr::Adsr,
    filter::{Filter, FilterTrait, FilterType},
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
    filter: Option<Filter>,
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
    filter_type: Option<FilterType>,
}

impl Synth {
    pub fn new(sr: f32, waveform: Waveform, filter_type: Option<FilterType>) -> Self {
        Self {
            sr,
            voices: [Voice::default(); MAX_VOICES],
            master_volume: 0.2,
            attack: 0.0,
            decay: 0.5,
            sustain: 1.0,
            release: 0.5,
            waveform,
            filter_type,
        }
    }

    pub fn note_on(&mut self, note: Note) {
        let mut adsr = Adsr::new(self.attack, self.decay, self.sustain, self.release, self.sr);
        adsr.note_on();
        if let Some(v) = self.voices.iter_mut().find(|v| v.on && v.note == note) {
            v.asdr = adsr;
            let _ = v.filter.as_mut().map(|f| f.reset());
            return;
        }
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.on) {
            voice.on = true;
            voice.note = note;
            voice.phase = 0.0;
            voice.asdr = adsr;
            voice.osc = Osc::new(note.into(), self.sr, self.waveform);
            voice.filter = self
                .filter_type
                .map(|filter_type| Filter::new(filter_type, self.sr));
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
                let osc_sample = voice
                    .filter
                    .as_mut()
                    .map(|f| f.process(osc_sample))
                    .unwrap_or(osc_sample);
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
                v.asdr.retune(a, d, s, r);
                if v.asdr.is_active() {
                    v.asdr.note_on();
                }
            }
        }
    }

    pub fn set_waveform(&mut self, new: Waveform) {
        self.waveform = new;
        for v in self.voices.iter_mut() {
            v.osc.set_waveform(new);
        }
    }

    pub fn set_filter(&mut self, new: Option<FilterType>) {
        self.filter_type = new;
        for v in self.voices.iter_mut() {
            match (new, v.filter.as_mut()) {
                (None, _) => {
                    v.filter = None;
                }
                (Some(ft), None) => {
                    v.filter = Some(Filter::new(ft, self.sr));
                }
                (Some(FilterType::OnePoleLpf(c)), Some(Filter::OnePoleLpf(f))) => {
                    f.set_cutoff(self.sr, c); // 型は同じ → 係数更新だけ
                }
                (Some(FilterType::TwoPoleLpf(c, q)), Some(Filter::TwoPoleLpf(f))) => {
                    f.set_params(self.sr, c, q);
                }
                (Some(ft), Some(_old_other_type)) => {
                    // 型が変わる → 作り直す（必要なら新規に reset 済み）
                    v.filter = Some(Filter::new(ft, self.sr));
                }
            }
        }
    }
}
