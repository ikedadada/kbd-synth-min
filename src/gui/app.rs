use crate::synth::{FilterType, Msg, Note, SharedBus, Waveform};
use eframe::{App, Frame, egui};

pub struct EguiUi {
    bus: SharedBus,
    master: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    waveform: WaveformUi,
    filter: FilterUi,
}

#[derive(Clone, PartialEq)]
pub enum WaveformTypeUi {
    Sine,
    Square,
    Saw,
    Triangle,
}

impl WaveformTypeUi {
    fn to_waveform(&self, pulse_width: f32, curve: f32) -> Waveform {
        match self {
            WaveformTypeUi::Sine => Waveform::Sine,
            WaveformTypeUi::Square => Waveform::Square { pulse_width },
            WaveformTypeUi::Saw => Waveform::Sawtooth,
            WaveformTypeUi::Triangle => Waveform::Triangle { curve },
        }
    }
}

#[derive(Clone)]
pub struct WaveformUi {
    pulse_width: f32,
    curve: f32,
    waveform_type: WaveformTypeUi,
}

impl From<WaveformUi> for Waveform {
    fn from(ui: WaveformUi) -> Self {
        ui.waveform_type.to_waveform(ui.pulse_width, ui.curve)
    }
}

#[derive(Clone, PartialEq)]
pub enum FilterTypeUi {
    OnePoleLpf,
    TwoPoleLpf,
}

impl FilterTypeUi {
    fn to_filter_type(&self, cut_off: f32, q: f32) -> FilterType {
        match self {
            Self::OnePoleLpf => FilterType::OnePoleLpf(cut_off),
            Self::TwoPoleLpf => FilterType::TwoPoleLpf(cut_off, q),
        }
    }
}

#[derive(Clone)]
pub struct FilterUi {
    show: bool,
    cutoff: f32,
    q: f32,
    filter_type: FilterTypeUi,
}

impl From<FilterUi> for FilterType {
    fn from(ui: FilterUi) -> Self {
        ui.filter_type.to_filter_type(ui.cutoff, ui.q)
    }
}

impl EguiUi {
    pub fn new(bus: SharedBus) -> Self {
        let ui = Self {
            bus,
            master: 0.2,
            attack: 0.0,
            decay: 0.5,
            sustain: 1.0,
            release: 0.5,
            waveform: WaveformUi {
                pulse_width: 0.5,
                curve: 0.0,
                waveform_type: WaveformTypeUi::Sine,
            },
            filter: FilterUi {
                show: false,
                cutoff: 1000.0,
                q: 5.0,
                filter_type: FilterTypeUi::OnePoleLpf,
            },
        };
        // Push initial params
        let _ = ui.bus.q.push(Msg::SetMasterVolume(ui.master));
        let _ = ui.bus.q.push(Msg::SetAdsr {
            a: ui.attack,
            d: ui.decay,
            s: ui.sustain,
            r: ui.release,
        });
        let _ = ui.bus.q.push(Msg::SetWaveform(ui.waveform.clone().into()));
        ui
    }
}

impl App for EguiUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Read current input events and whether UI wants keyboard focus
        let events = ctx.input(|i| i.events.clone());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Synth Controls");
            let mut changed = (false, false, false, false);
            changed.0 |= ui
                .add(egui::Slider::new(&mut self.master, 0.0..=1.0).text("Master"))
                .changed();
            changed.1 |= ui
                .add(egui::Slider::new(&mut self.attack, 0.0..=2.0).text("Attack"))
                .changed();
            changed.1 |= ui
                .add(egui::Slider::new(&mut self.decay, 0.0..=2.0).text("Decay"))
                .changed();
            changed.1 |= ui
                .add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain"))
                .changed();
            changed.1 |= ui
                .add(egui::Slider::new(&mut self.release, 0.0..=2.0).text("Release"))
                .changed();

            ui.horizontal(|ui| {
                ui.label("Waveform:");
                changed.2 |= ui
                    .selectable_value(
                        &mut self.waveform.waveform_type,
                        WaveformTypeUi::Sine,
                        "Sine",
                    )
                    .changed();
                changed.2 |= ui
                    .selectable_value(
                        &mut self.waveform.waveform_type,
                        WaveformTypeUi::Square,
                        "Square",
                    )
                    .changed();
                changed.2 |= ui
                    .selectable_value(&mut self.waveform.waveform_type, WaveformTypeUi::Saw, "Saw")
                    .changed();
                changed.2 |= ui
                    .selectable_value(
                        &mut self.waveform.waveform_type,
                        WaveformTypeUi::Triangle,
                        "Tri",
                    )
                    .changed();
            });

            // ここを追加: 選択中の波形に応じたパラメータUI
            match self.waveform.waveform_type {
                WaveformTypeUi::Square => {
                    ui.horizontal(|ui| {
                        ui.label("Pulse width:");
                        // 0や1は無音/直流に近くなるので少しマージンを取るのが無難
                        changed.2 |= ui
                            .add(
                                egui::Slider::new(&mut self.waveform.pulse_width, 0.05..=0.95)
                                    .text("PW"),
                            )
                            .changed();
                    });
                }
                WaveformTypeUi::Triangle => {
                    ui.horizontal(|ui| {
                        ui.label("Curve:");
                        // 0.0 = リニア、1.0 で尖りが強くなる想定
                        changed.2 |= ui
                            .add(
                                egui::Slider::new(&mut self.waveform.curve, 0.0..=1.0)
                                    .text("Curve"),
                            )
                            .changed();
                    });
                }
                _ => {}
            }

            // フィルタ選択UIの追加
            // フィルタのOn/Off
            ui.label("Filter:");
            // FilterのOn/Off
            changed.3 |= ui.checkbox(&mut self.filter.show, "Enabled").changed();
            if self.filter.show {
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    changed.3 |= ui
                        .selectable_value(
                            &mut self.filter.filter_type,
                            FilterTypeUi::OnePoleLpf,
                            "OnePoleLpf",
                        )
                        .changed();
                    changed.3 |= ui
                        .selectable_value(
                            &mut self.filter.filter_type,
                            FilterTypeUi::TwoPoleLpf,
                            "TwoPoleLpf",
                        )
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Cutoff:");
                    changed.3 |= ui
                        .add(
                            egui::Slider::new(&mut self.filter.cutoff, 20.0..=20000.0)
                                .text("Cutoff"),
                        )
                        .changed();
                });
                if self.filter.filter_type == FilterTypeUi::TwoPoleLpf {
                    ui.horizontal(|ui| {
                        ui.label("Resonance (Q):");
                        changed.3 |= ui
                            .add(egui::Slider::new(&mut self.filter.q, 0.1..=10.0).text("Q"))
                            .changed();
                    });
                }
            };

            if changed.0 {
                let _ = self.bus.q.push(Msg::SetMasterVolume(self.master));
            }
            if changed.1 {
                let _ = self.bus.q.push(Msg::SetAdsr {
                    a: self.attack,
                    d: self.decay,
                    s: self.sustain,
                    r: self.release,
                });
            }
            if changed.2 {
                let _ = self
                    .bus
                    .q
                    .push(Msg::SetWaveform(self.waveform.clone().into()));
            }
            if changed.3 {
                let filter_msg = if self.filter.show {
                    Some(self.filter.clone().into())
                } else {
                    None
                };
                let _ = self.bus.q.push(Msg::SetFilter(filter_msg));
            }
        });

        // Global keyboard handling (when UI doesn't want text input)
        for ev in events {
            if let egui::Event::Key {
                key,
                pressed,
                repeat,
                ..
            } = ev
            {
                if repeat {
                    continue;
                }
                let note = key.into();
                if note == Note::None {
                    continue;
                }
                let _ = if pressed {
                    self.bus.q.push(Msg::NoteOn { note })
                } else {
                    self.bus.q.push(Msg::NoteOff { note })
                };
            }
        }

        // Render ~30 FPS
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}

impl From<egui::Key> for Note {
    fn from(key: egui::Key) -> Note {
        use egui::Key::*;
        match key {
            A => Note::C4,
            W => Note::C4_5,
            S => Note::D4,
            E => Note::D4_5,
            D => Note::E4,
            F => Note::F4,
            T => Note::F4_5,
            G => Note::G4,
            Y => Note::G4_5,
            H => Note::A4,
            U => Note::A4_5,
            J => Note::B4,
            K => Note::C5,
            O => Note::C5_5,
            L => Note::D5,
            P => Note::D5_5,
            Semicolon => Note::E5,
            Colon => Note::F5,
            OpenBracket => Note::F5_5,
            CloseBracket => Note::G5,
            _ => {
                println!("Unhandled key: {:?}", key);
                Note::None
            }
        }
    }
}
