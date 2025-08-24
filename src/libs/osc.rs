use std::f32::consts::TAU;

#[derive(Debug, Clone)]
pub struct Osc {
    phase: f32,
    amp: f32,
    waveform: Waveform,
    phase_inc: f32,
}

impl Osc {
    pub fn new(freq_hz: f32, sr: f32, waveform: Waveform) -> Self {
        let phase = 0.0;
        let amp = 1.0;
        let phase_inc = (freq_hz / sr) * TAU; // 1周期を2πとする
        Self {
            phase,
            amp,
            waveform,
            phase_inc,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = self.waveform.sample(self.phase) * self.amp;
        self.phase += self.phase_inc;
        if self.phase >= TAU {
            self.phase -= TAU;
        }
        sample
    }
}

#[derive(Debug, Clone)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

impl Waveform {
    pub fn sample(&self, phase: f32) -> f32 {
        match self {
            Waveform::Sine => phase.sin(),
            Waveform::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sawtooth => phase / TAU * 2.0 - 1.0,
            Waveform::Triangle => {
                let t = phase / TAU;
                1.0 - 4.0 * (t - 0.5).abs()
            }
        }
    }
}
