use crate::synth::{FilterType, Msg, Note, SharedBus, Waveform};
use eframe::{App, Frame, egui};

pub struct EguiUi {
    bus: SharedBus,
    master: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    waveform: Waveform,
    filter: FilterUi,
}

#[derive(PartialEq)]
pub enum FilterTypeUi {
    OnePoleLpf,
    TwoPoleLpf,
}

impl FilterTypeUi {
    fn to_filter_type(&self, cut_off: f32) -> FilterType {
        match self {
            Self::OnePoleLpf => FilterType::OnePoleLpf(cut_off),
            Self::TwoPoleLpf => FilterType::TwoPoleLpf(cut_off),
        }
    }
}

pub struct FilterUi {
    show: bool,
    cutoff: f32,
    filter_type: FilterTypeUi,
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
            waveform: Waveform::Sine,
            filter: FilterUi {
                show: false,
                cutoff: 1000.0,
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
        let _ = ui.bus.q.push(Msg::SetWaveform(ui.waveform));
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
                    .selectable_value(&mut self.waveform, Waveform::Sine, "Sine")
                    .changed();
                changed.2 |= ui
                    .selectable_value(&mut self.waveform, Waveform::Square, "Square")
                    .changed();
                changed.2 |= ui
                    .selectable_value(&mut self.waveform, Waveform::Sawtooth, "Saw")
                    .changed();
                changed.2 |= ui
                    .selectable_value(&mut self.waveform, Waveform::Triangle, "Tri")
                    .changed();
            });

            // フィルタ選択UIの追加
            // フィルタのOn/Off
            ui.label("Filter:");
            // FilterのOn/Off
            changed.3 |= ui.checkbox(&mut self.filter.show, "Enabled").changed();
            if self.filter.show {
                ui.horizontal(|ui| {
                    ui.label("Cutoff:");
                    changed.3 |= ui
                        .add(
                            egui::Slider::new(&mut self.filter.cutoff, 20.0..=20000.0)
                                .text("Cutoff"),
                        )
                        .changed();
                });
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
                let _ = self.bus.q.push(Msg::SetWaveform(self.waveform));
            }
            if changed.3 {
                let filter_msg = if self.filter.show {
                    Some(self.filter.filter_type.to_filter_type(self.filter.cutoff))
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
            Z => Note::C4,
            X => Note::D4,
            C => Note::E4,
            V => Note::F4,
            B => Note::G4,
            N => Note::A4,
            M => Note::B4,
            Comma => Note::C5,
            _ => Note::None,
        }
    }
}
