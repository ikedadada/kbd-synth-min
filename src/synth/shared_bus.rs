use std::sync::Arc;

use crossbeam::queue::ArrayQueue;

use crate::synth::{Note, osc::Waveform};

const QUEUE_CAP: usize = 2048;

#[derive(Debug)]
pub enum Msg {
    NoteOn { note: Note },
    NoteOff { note: Note },
    SetMasterVolume(f32),
    SetAdsr { a: f32, d: f32, s: f32, r: f32 },
    SetWaveform(Waveform),
}

#[derive(Clone, Debug)]
pub struct SharedBus {
    pub q: Arc<ArrayQueue<Msg>>,
}

impl Default for SharedBus {
    fn default() -> Self {
        let q = Arc::new(ArrayQueue::new(QUEUE_CAP));
        Self { q }
    }
}
