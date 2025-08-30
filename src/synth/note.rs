#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Note {
    #[default]
    None,
    C4,
    D4,
    E4,
    F4,
    G4,
    A4,
    B4,
    C5,
}

impl From<Note> for f32 {
    fn from(note: Note) -> Self {
        match note {
            Note::None => 0.0,
            Note::C4 => 261.626,
            Note::D4 => 293.665,
            Note::E4 => 329.628,
            Note::F4 => 349.228,
            Note::G4 => 392.0,
            Note::A4 => 440.0,
            Note::B4 => 493.883,
            Note::C5 => 523.251,
        }
    }
}
