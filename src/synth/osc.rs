use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, Default)]
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
        let sample = self.waveform.sample(self.phase, self.phase_inc);
        self.phase += self.phase_inc;
        if self.phase >= TAU {
            self.phase -= TAU;
        }
        sample * self.amp
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Waveform {
    #[default]
    Sine,
    Square {
        pulse_width: f32,
    },
    Sawtooth,
    Triangle {
        curve: f32,
    },
}

impl Waveform {
    pub fn sample(&self, phase: f32, phase_inc: f32) -> f32 {
        const BAND_WIDTH: f32 = 1.0;
        let t = phase / TAU;
        let dt = phase_inc / TAU;
        match self {
            Waveform::Sine => phase.sin(),
            Waveform::Square { pulse_width } => {
                let p = pulse_width.clamp(0.05, 0.95);
                let mut y = if t < p { 1.0 } else { -1.0 };
                y += BAND_WIDTH * poly_blep(t, dt); // 上がり（t=0）
                y -= BAND_WIDTH * poly_blep((t - p).rem_euclid(1.0), dt); // 下がり（t=p）
                y
            }
            Waveform::Sawtooth => {
                let mut y = 2.0 * t - 1.0;
                y -= BAND_WIDTH * poly_blep(t, dt);
                y
            }
            Waveform::Triangle { curve } => {
                // BAND_WIDTHの適用は効果が薄かったため省略
                let tri = 1.0 - 4.0 * (t - 0.5).abs();
                tri.signum() * tri.abs().powf(1.0 + *curve * 2.0)
            }
        }
    }
}

/* ---------------- ヘルパ ---------------- */
#[inline]
fn poly_blep(mut t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    if t < dt {
        t /= dt;
        t + t - t * t - 1.0 // 2t - t^2 - 1
    } else if t > 1.0 - dt {
        t = (t - 1.0) / dt;
        t * t + t + t + 1.0 // t^2 + 2t + 1
    } else {
        0.0
    }
}
