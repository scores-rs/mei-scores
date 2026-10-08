//! Read and write MEI (Music Encoding Initiative) documents.
//!
//! [MEI](https://music-encoding.org) is an XML format for music notation,
//! defined by its guidelines as modules of elements (`MEI.header`,
//! `MEI.shared`, `MEI.cmn`, ...) whose attributes draw on shared
//! vocabularies (`data.DURATION`, `data.PITCHNAME`, ...). This crate
//! models the part of MEI 5 that common practice notation needs, as
//! typed records named after the elements and attributes:
//!
//! - the header's title statement (`<meiHead>`, [`header`]);
//! - the score's structure, from `<music>` through `<mdiv>`, `<score>`,
//!   `<scoreDef>`/`<staffGrp>`/`<staffDef>` and `<section>`/`<ending>` to
//!   `<staff>` and `<layer>`, and the events in a layer: `<note>`,
//!   `<rest>`, `<chord>`, clef/key/meter changes, `<artic>`, `<dynam>`,
//!   `<dir>` and `<ornam>` ([`shared`]);
//! - measured notation: `<measure>` with its barlines, `<beam>`,
//!   `<tuplet>`, `<hairpin>`, `<fermata>` and MEI 5's `<repeatMark>`
//!   ([`cmn`]); chord symbols (`<harm>`, [`harmony`]); trills, mordents and
//!   turns ([`ornaments`]);
//! - the attribute vocabularies ([`data`]) and attribute classes ([`att`]).
//!
//! Anything else is not modeled. Reading is lenient: an unmodeled element
//! is looked through, so the modeled elements inside it (a `<note>` in an
//! `<app>`, a `<layer>` in a `<choice>`) are still read, and attribute
//! values outside MEI's vocabularies are kept as written. A few
//! non-standard forms that the `scores` converter writes are modeled too
//! (documented where they appear): a `<staffDef>` `@label`, a
//! `@repeat.count` on `<measure>`, control events written inside a layer
//! or note, and a legacy `<navigation>` element.
//!
//! # Reading
//!
//! ```
//! use mei_scores::Mei;
//! use mei_scores::cmn::MeasureElement;
//! use mei_scores::shared::{LayerElement, SectionElement, StaffElement};
//!
//! let xml = r#"<mei xmlns="http://www.music-encoding.org/ns/mei" meiversion="5.0">
//!   <music><body><mdiv><score><section><measure n="1">
//!     <staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff>
//!   </measure></section></score></mdiv></body></music>
//! </mei>"#;
//! let mei = Mei::from_bytes(xml.as_bytes())?;
//! let score = mei.music.body.mdivs[0].score.as_ref().unwrap();
//! let SectionElement::Measure(measure) = &score.sections[0].content[0] else { panic!() };
//! let MeasureElement::Staff(staff) = &measure.content[0] else { panic!() };
//! let StaffElement::Layer(layer) = &staff.content[0] else { panic!() };
//! let LayerElement::Note(note) = &layer.content[0] else { panic!() };
//! assert_eq!(note.oct, Some(4));
//! # Ok::<(), mei_scores::MeiError>(())
//! ```
//!
//! # Writing
//!
//! ```
//! use mei_scores::Mei;
//! use mei_scores::header::{MeiHead, TitleStmt};
//!
//! let mut mei = Mei::new();
//! let mut head = MeiHead::default();
//! head.file_desc.title_stmt = TitleStmt {
//!     title: Some("Scale".into()),
//!     composer: None,
//! };
//! mei.head = Some(head);
//! let xml = String::from_utf8(mei.to_bytes()?).unwrap();
//! assert!(xml.contains("<title>Scale</title>"));
//! # Ok::<(), mei_scores::MeiError>(())
//! ```

pub mod att;
pub mod cmn;
pub mod data;
mod error;
pub mod harmony;
pub mod header;
pub mod ornaments;
mod read;
pub mod shared;
mod write;

pub use error::MeiError;

use header::MeiHead;
use shared::Music;

/// MEI's XML namespace.
pub const NAMESPACE: &str = "http://www.music-encoding.org/ns/mei";

/// The MEI version written by [`Mei::new`].
pub const VERSION: &str = "5.0";

/// An MEI document: `<mei>`, its header and its music.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mei {
    /// `@meiversion`.
    pub meiversion: Option<String>,
    pub head: Option<MeiHead>,
    pub music: Music,
}

impl Mei {
    /// An empty MEI 5 document.
    pub fn new() -> Self {
        Self {
            meiversion: Some(VERSION.to_string()),
            ..Self::default()
        }
    }

    /// Reads a document from UTF-8 MEI XML.
    pub fn from_bytes(data: &[u8]) -> Result<Self, MeiError> {
        read::read(data)
    }

    /// Writes the document as MEI XML, with an XML declaration and two
    /// spaces of indentation.
    pub fn to_bytes(&self) -> Result<Vec<u8>, MeiError> {
        write::write(self)
    }
}
