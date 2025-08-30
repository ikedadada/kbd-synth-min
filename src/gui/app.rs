use crate::synth::{Msg, Note, SharedBus, Waveform};
use eframe::{App, Frame, egui};

pub struct EguiUi {
    bus: SharedBus,
    master: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    waveform: Waveform,
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
            let mut changed = false;
            changed |= ui
                .add(egui::Slider::new(&mut self.master, 0.0..=1.0).text("Master"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.attack, 0.0..=2.0).text("Attack"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.decay, 0.0..=2.0).text("Decay"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.release, 0.0..=2.0).text("Release"))
                .changed();

            ui.horizontal(|ui| {
                ui.label("Waveform:");
                changed |= ui
                    .selectable_value(&mut self.waveform, Waveform::Sine, "Sine")
                    .changed();
                changed |= ui
                    .selectable_value(&mut self.waveform, Waveform::Square, "Square")
                    .changed();
                changed |= ui
                    .selectable_value(&mut self.waveform, Waveform::Sawtooth, "Saw")
                    .changed();
                changed |= ui
                    .selectable_value(&mut self.waveform, Waveform::Triangle, "Tri")
                    .changed();
            });

            if changed {
                let _ = self.bus.q.push(Msg::SetMasterVolume(self.master));
                let _ = self.bus.q.push(Msg::SetAdsr {
                    a: self.attack,
                    d: self.decay,
                    s: self.sustain,
                    r: self.release,
                });
                let _ = self.bus.q.push(Msg::SetWaveform(self.waveform));
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
