#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    OnePoleLpf(f32),      // カットオフ周波数
    TwoPoleLpf(f32, f32), // カットオフ周波数とレゾナンス
}

#[derive(Clone, Copy)]
pub enum Filter {
    OnePoleLpf(OnePoleLpf),
    TwoPoleLpf(TwoPoleLpf),
}

impl Filter {
    pub fn new(filter_type: FilterType, sr: f32) -> Self {
        match filter_type {
            FilterType::OnePoleLpf(cutoff) => Filter::OnePoleLpf(OnePoleLpf::new(sr, cutoff)),
            FilterType::TwoPoleLpf(cutoff, q) => Filter::TwoPoleLpf(TwoPoleLpf::new(sr, cutoff, q)),
        }
    }
}

pub trait FilterTrait {
    fn process(&mut self, input: f32) -> f32;
    fn reset(&mut self);
}

impl FilterTrait for Filter {
    fn process(&mut self, x: f32) -> f32 {
        match self {
            Filter::OnePoleLpf(f) => f.process(x),
            Filter::TwoPoleLpf(f) => f.process(x),
        }
    }
    fn reset(&mut self) {
        match self {
            Filter::OnePoleLpf(f) => f.reset(),
            Filter::TwoPoleLpf(f) => f.reset(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct OnePoleLpf {
    cutoff: f32, // カットオフ周波数 [0 ~ sr/2]
    a: f32,      // フィルタ係数
    y1: f32,     // 前回の出力
}

impl OnePoleLpf {
    pub fn new(sr: f32, cutoff: f32) -> Self {
        let mut f = Self {
            cutoff,
            a: 0.0,
            y1: 0.0,
        };
        f.set_cutoff(sr, cutoff);
        f
    }
}

impl OnePoleLpf {
    pub fn set_cutoff(&mut self, sr: f32, cutoff: f32) {
        self.cutoff = cutoff.clamp(0.0, 0.49 * sr);
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.cutoff.max(1e-6));
        let dt = 1.0 / sr;
        self.a = dt / (rc + dt);
    }
}

impl FilterTrait for OnePoleLpf {
    fn process(&mut self, input: f32) -> f32 {
        self.y1 += self.a * (input - self.y1);
        self.y1
    }

    fn reset(&mut self) {
        self.y1 = 0.0;
    }
}

#[derive(Clone, Copy, Default)]
pub struct TwoPoleLpf {
    cutoff: f32,
    q: f32, // レゾナンス
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32, // フィルタ係数
    y1: f32,
    y2: f32, // 前回の出力
}

impl TwoPoleLpf {
    pub fn new(sr: f32, cutoff: f32, q: f32) -> Self {
        let mut f = Self {
            cutoff,
            q,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        f.update_coefficients(sr);
        f
    }

    pub fn set_params(&mut self, sr: f32, cutoff: f32, q: f32) {
        self.cutoff = cutoff;
        self.q = q;
        self.update_coefficients(sr);
    }

    pub fn update_coefficients(&mut self, sr: f32) {
        let f0 = self.cutoff.clamp(1.0, 0.49 * sr);
        let w0 = 2.0 * std::f32::consts::PI * f0 / sr;
        let (sw, cw0) = w0.sin_cos();
        let alpha = sw / (2.0 * self.q.max(1e-6));

        let b0 = (1.0 - cw0) / 2.0;
        let b1 = 1.0 - cw0;
        let b2 = (1.0 - cw0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cw0;
        let a2 = 1.0 - alpha;

        self.b0 = b0 / a0; // 正規化された係数
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl FilterTrait for TwoPoleLpf {
    fn process(&mut self, input: f32) -> f32 {
        let y = self.b0 * input + self.y1;
        self.y1 = self.b1 * input - self.a1 * y + self.y2;
        self.y2 = self.b2 * input - self.a2 * y;
        y
    }

    fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}
