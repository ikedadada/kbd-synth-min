#[derive(Debug, Clone, Copy, Default)]
pub enum EnvState {
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Adsr {
    a: f32, // Attack time
    d: f32, // Decay time
    s: f32, // Sustain level
    r: f32, // Release time

    state: EnvState,

    sr: f32, // Sample rate
    level: f32,
    gate: bool,

    target: f32,
    step: f32,
}

impl Adsr {
    pub fn new(a: f32, d: f32, s: f32, r: f32, sr: f32) -> Self {
        Self {
            a,
            d,
            s: s.clamp(0.0, 1.0),
            r,
            sr,
            state: EnvState::Idle,
            level: 0.0,
            gate: false,
            target: 0.0,
            step: 0.0,
        }
    }

    pub fn retune(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.a = a;
        self.d = d;
        self.s = s.clamp(0.0, 1.0);
        self.r = r;
    }

    #[inline]
    fn set_stage(&mut self, time_sec: f32, target: f32) {
        self.target = target;
        if time_sec <= 0.0 || (self.level - target).abs() < f32::EPSILON {
            self.level = target;
            self.step = 0.0;
            self.advance_after_hit();
        } else {
            self.step = (target - self.level) / (time_sec * self.sr);
        }
    }

    #[inline]
    fn enter_attack(&mut self) {
        self.state = EnvState::Attack;
        self.set_stage(self.a, 1.0);
    }
    #[inline]
    fn enter_decay(&mut self) {
        self.state = EnvState::Decay;
        self.set_stage(self.d, self.s);
    }
    #[inline]
    fn enter_sustain(&mut self) {
        self.state = EnvState::Sustain;
        self.level = self.s;
        self.target = self.s;
        self.step = 0.0;
    }
    #[inline]
    fn enter_release(&mut self) {
        self.state = EnvState::Release;
        self.set_stage(self.r, 0.0);
    }

    #[inline]
    fn advance_after_hit(&mut self) {
        match self.state {
            EnvState::Attack => {
                // ノートオフ済みならDecayを飛ばしてReleaseへ
                if self.gate {
                    self.enter_decay();
                } else {
                    self.enter_release();
                }
            }
            EnvState::Decay => {
                if self.gate {
                    self.enter_sustain();
                } else {
                    self.enter_release();
                }
            }
            EnvState::Release => {
                self.state = EnvState::Idle;
                self.level = 0.0;
            }
            EnvState::Sustain | EnvState::Idle => {}
        }
    }

    pub fn note_on(&mut self) {
        self.gate = true;
        self.enter_attack();
    }
    pub fn note_off(&mut self) {
        self.gate = false;
        self.enter_release();
    }

    /// 1サンプル進めて現在値を返す
    pub fn next_sample(&mut self) -> f32 {
        match self.state {
            EnvState::Idle => {
                self.level = 0.0;
            }
            EnvState::Sustain => {
                self.level = self.s;
                if !self.gate {
                    self.enter_release();
                }
            }
            EnvState::Attack | EnvState::Decay | EnvState::Release => {
                self.level += self.step;
                let hit = if self.step >= 0.0 {
                    self.level >= self.target
                } else {
                    self.level <= self.target
                };
                if hit {
                    self.level = self.target;
                    self.advance_after_hit();
                }
            }
        }
        // デノーマル対策（超微小値を0へ）
        if self.level.abs() < 1.0e-12 {
            self.level = 0.0;
        }
        self.level
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.level > 0.0
    }
}
