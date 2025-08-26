use std::sync::Arc;

use crossbeam::queue::ArrayQueue;

const QUEUE_CAP: usize = 2048;

pub enum Msg {
    NoteOn { note: f32 },
    NoteOff { note: f32 },
}

#[derive(Clone)]
pub struct SharedBus {
    pub q: Arc<ArrayQueue<Msg>>,
}

impl SharedBus {
    pub fn new() -> Self {
        let q = Arc::new(ArrayQueue::new(QUEUE_CAP));
        Self { q }
    }
}

pub fn handle_key_down(bus: &SharedBus, note: f32) {
    let _ = bus.q.push(Msg::NoteOn { note });
}

pub fn handle_key_up(bus: &SharedBus, note: f32) {
    let _ = bus.q.push(Msg::NoteOff { note });
}
