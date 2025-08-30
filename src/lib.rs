pub mod synth {
    mod adsr;
    mod engine;
    mod note;
    mod osc;
    mod shared_bus;
    // Re-export primary types to avoid deep paths
    pub use engine::Synth;
    pub use note::Note;
    pub use osc::Waveform;
    pub use shared_bus::Msg;
    pub use shared_bus::SharedBus;
}

pub mod gui {
    mod app;
    pub use app::EguiUi;
}
