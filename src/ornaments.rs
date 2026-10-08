//! CMN ornaments (MEI.cmnOrnaments): trills, mordents and turns.

use crate::att;
use crate::data::OrnamentForm;

/// `<trill>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trill {
    pub control: att::ControlEvent,
}

/// `<mordent>`: `lower` is the plain mordent, `upper` the inverted one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mordent {
    pub control: att::ControlEvent,
    pub form: Option<OrnamentForm>,
}

/// `<turn>`: `upper` starts on the note above (the usual turn), `lower` on
/// the one below (an inverted turn).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Turn {
    pub control: att::ControlEvent,
    pub form: Option<OrnamentForm>,
}
