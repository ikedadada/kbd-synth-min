#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    OnePoleLpf(f32), // カットオフ周波数
    TwoPoleLpf(f32),
}

#[derive(Clone, Copy)]
pub enum Filter {
    OnePoleLpf(OnePoleLpf),
    TwoPoleLpf(TwoPoleLpf),
}

impl Default for Filter {
    fn default() -> Self {
        Self::OnePoleLpf(OnePoleLpf::new(44100.0, 2000.0))
    }
}

impl Filter {
    pub fn new(filter_type: FilterType, sr: f32) -> Self {
        match filter_type {
            FilterType::OnePoleLpf(cutoff) => Filter::OnePoleLpf(OnePoleLpf::new(sr, cutoff)),
            FilterType::TwoPoleLpf(cutoff) => Filter::TwoPoleLpf(TwoPoleLpf::new(sr, cutoff)),
        }
    }
}

pub trait FilterTrait {
    fn process(&mut self, input: f32) -> f32;
    fn set_cutoff(&mut self, sr: f32, cutoff: f32);
    fn reset(&mut self);
}

impl FilterTrait for Filter {
    fn process(&mut self, x: f32) -> f32 {
        match self {
            Filter::OnePoleLpf(f) => f.process(x),
            Filter::TwoPoleLpf(f) => f.process(x),
        }
    }
    fn set_cutoff(&mut self, sr: f32, c: f32) {
        match self {
            Filter::OnePoleLpf(f) => f.set_cutoff(sr, c),
            Filter::TwoPoleLpf(f) => f.set_cutoff(sr, c),
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

impl FilterTrait for OnePoleLpf {
    fn set_cutoff(&mut self, sr: f32, cutoff: f32) {
        self.cutoff = cutoff;
        self.a = 2.0 * std::f32::consts::PI * cutoff / sr;
    }

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
    a: f32,  // フィルタ係数
    y1: f32, // 前回の出力
    y2: f32, // 2回前の出力
}

impl TwoPoleLpf {
    pub fn new(sr: f32, cutoff: f32) -> Self {
        let mut f = Self {
            cutoff,
            a: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        f.set_cutoff(sr, cutoff);
        f
    }
}

impl FilterTrait for TwoPoleLpf {
    fn set_cutoff(&mut self, sr: f32, cutoff: f32) {
        self.cutoff = cutoff;
        self.a = 2.0 * std::f32::consts::PI * cutoff / sr;
    }

    fn process(&mut self, input: f32) -> f32 {
        let y = self.y1 + self.a * (input - self.y1) + self.a * (self.y1 - self.y2);
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}
