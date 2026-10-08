//! Common Music Notation (MEI.cmn): measures, beams, tuplets, and the
//! control events of measured notation.

use crate::att;
use crate::data::{BarRendition, HairpinForm, Place, RepeatMarkFunc};
use crate::shared::{ControlElement, LayerElement, Staff};

/// `<measure>`: the staves' content between two barlines, followed by the
/// measure's control events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Measure {
    pub n: Option<u32>,
    /// The barline before the measure.
    pub left: Option<BarRendition>,
    /// The barline after the measure.
    pub right: Option<BarRendition>,
    /// `@repeat.count`: how often a `rptend` repeat is played, when not
    /// twice. Not MEI; written by `scores`.
    pub repeat_count: Option<u32>,
    pub content: Vec<MeasureElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MeasureElement {
    Staff(Staff),
    Control(ControlElement),
}

/// `<beam>`: beamed events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Beam {
    pub content: Vec<LayerElement>,
}

/// `<tuplet>`: `num` events in the time of `numbase`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tuplet {
    pub num: Option<u8>,
    pub numbase: Option<u8>,
    pub content: Vec<LayerElement>,
}

/// `<hairpin>`: a crescendo or diminuendo wedge, from `@startid`/`@tstamp`
/// to `@endid`/`@tstamp2`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Hairpin {
    pub control: att::ControlEvent,
    /// `cres` or `dim`; files written by `scores` mark a wedge's end with
    /// a hairpin of form `end`.
    pub form: Option<HairpinForm>,
    pub endid: Option<String>,
    /// Where it ends: measures ahead and beat, `<m>m+<beat>`.
    pub tstamp2: Option<String>,
}

impl Hairpin {
    /// `@endid` without its leading `#`.
    pub fn end_id(&self) -> Option<&str> {
        self.endid.as_deref().map(|id| id.trim_start_matches('#'))
    }
}

/// `<fermata>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fermata {
    pub control: att::ControlEvent,
    pub place: Option<Place>,
}

/// `<repeatMark>` (MEI 5): a segno, coda, D.C., D.S. or Fine, with its
/// text.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RepeatMark {
    pub control: att::ControlEvent,
    pub func: Option<RepeatMarkFunc>,
    pub text: String,
}
