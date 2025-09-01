use crate::synth::{Msg, SharedBus, Synth};

pub const QUANTUM: usize = 128;

/// Render a mono block into `out`, draining pending bus messages first.
/// Applies simple clamping to avoid clipping.
pub fn render_block(synth: &mut Synth, bus: &SharedBus, out: &mut [f32]) {
    // Drain control messages once per block
    while let Some(msg) = bus.q.pop() {
        match msg {
            Msg::NoteOn { note } => synth.note_on(note),
            Msg::NoteOff { note } => synth.note_off(note),
            Msg::SetMasterVolume(v) => synth.set_master_volume(v),
            Msg::SetAdsr { a, d, s, r } => synth.set_adsr(a, d, s, r),
            Msg::SetWaveform(wf) => synth.set_waveform(wf),
            Msg::SetFilter(ft) => synth.set_filter(ft),
        }
    }

    for s in out.iter_mut() {
        let v = synth.next_sample();
        *s = v.clamp(-1.0, 1.0);
    }
}

