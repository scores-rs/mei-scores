//! Harmony (MEI.harmony): chord symbols.

use crate::att;

/// `<harm>`: a harmony indication such as a chord symbol, as text
/// (`Cmaj7`, `F#m7b5/E`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Harm {
    pub control: att::ControlEvent,
    pub text: String,
}
