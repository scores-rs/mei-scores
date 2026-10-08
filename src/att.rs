//! Attribute classes (`att.*`) several elements share.

/// `att.controlEvent`: what a control event applies to - an event by its
/// `xml:id` (`@startid`), or a beat on a staff (`@tstamp`, `@staff`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ControlEvent {
    /// A reference to the event, as written (`#n1`).
    pub startid: Option<String>,
    /// The beat, counted in the meter's unit from 1.
    pub tstamp: Option<f64>,
    /// The staves it applies to.
    pub staff: Vec<u32>,
}

impl ControlEvent {
    /// `@startid` without its leading `#`.
    pub fn start_id(&self) -> Option<&str> {
        self.startid.as_deref().map(|id| id.trim_start_matches('#'))
    }
}
