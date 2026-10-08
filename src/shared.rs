//! The shared module (MEI.shared): the score's structure and the events
//! common to all repertoires - from `<music>` down to `<note>` and
//! `<rest>`, plus the clef/key/meter elements and the text directives.

use crate::att;
use crate::cmn::{Beam, Fermata, Hairpin, Measure, RepeatMark, Tuplet};
use crate::data::{Accidental, ClefShape, Duration, Grace, KeySig, Mode, PitchName, Place, Tie};
use crate::harmony::Harm;
use crate::ornaments::{Mordent, Trill, Turn};

/// `<music>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Music {
    pub body: Body,
}

/// `<body>`: the music's divisions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Body {
    pub mdivs: Vec<Mdiv>,
}

/// `<mdiv>`: a musical division (a movement, an act, ...).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mdiv {
    pub score: Option<Score>,
    /// Nested divisions.
    pub mdivs: Vec<Mdiv>,
}

/// `<score>`: a full score - its initial definitions and its sections.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Score {
    pub score_def: Option<ScoreDef>,
    pub sections: Vec<Section>,
}

/// `<scoreDef>`: the staves and their initial settings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScoreDef {
    pub staff_grp: StaffGrp,
}

/// `<staffGrp>`: a group of staves (a bracketed system, a piano's grand
/// staff), possibly nested.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StaffGrp {
    pub content: Vec<StaffGrpElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StaffGrpElement {
    StaffDef(StaffDef),
    StaffGrp(StaffGrp),
}

impl StaffGrp {
    /// Every `<staffDef>` in the group, nested groups included, in order.
    pub fn staff_defs(&self) -> Vec<&StaffDef> {
        let mut out = Vec::new();
        for element in &self.content {
            match element {
                StaffGrpElement::StaffDef(def) => out.push(def),
                StaffGrpElement::StaffGrp(group) => out.extend(group.staff_defs()),
            }
        }
        out
    }
}

/// `<staffDef>`: a staff's number, label, initial clef, key and meter, and
/// transposition, given as attributes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StaffDef {
    pub n: Option<u32>,
    pub lines: Option<u32>,
    /// The staff's name. Real MEI uses a `<label>` child element; this
    /// attribute form is what files written by `scores` use.
    pub label: Option<String>,
    pub clef_shape: Option<ClefShape>,
    pub clef_line: Option<i8>,
    /// `@clef.dis`: an octave mark's interval (`8`, `15`, `22`).
    pub clef_dis: Option<u8>,
    pub clef_dis_place: Option<Place>,
    pub key_sig: Option<KeySig>,
    pub key_mode: Option<Mode>,
    pub meter_count: Option<u8>,
    pub meter_unit: Option<u8>,
    /// `@trans.diat`: a transposing instrument's interval from written to
    /// sounding pitch in diatonic steps (`-1` for a B♭ clarinet).
    pub trans_diat: Option<i8>,
    /// `@trans.semi`: the same interval in semitones (`-2` for a B♭
    /// clarinet).
    pub trans_semi: Option<i8>,
}

/// `<section>`: a run of measures, possibly with endings, nested sections
/// and changed score definitions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Section {
    pub content: Vec<SectionElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionElement {
    Measure(Measure),
    Ending(Ending),
    ScoreDef(ScoreDef),
    Section(Section),
}

/// `<ending>`: a 1st/2nd (etc.) ending bracket wrapping its measures.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ending {
    /// Which passes the ending is for, as written (`1`, `1,2`).
    pub n: Option<String>,
    /// The bracket's end: `angledown` for a closed bracket, `none` for an
    /// open one.
    pub lendsym: Option<String>,
    /// `@type`, which older files written by `scores` set to
    /// `discontinue` for an open ending.
    pub r#type: Option<String>,
    pub content: Vec<SectionElement>,
}

/// `<staff>`: one staff's content in a measure.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Staff {
    pub n: Option<u32>,
    pub content: Vec<StaffElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StaffElement {
    Layer(Layer),
    Clef(Clef),
    KeySig(KeySigElement),
    MeterSig(MeterSig),
    Control(ControlElement),
}

/// `<layer>`: one voice of a staff, its events in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layer {
    pub n: Option<u8>,
    pub content: Vec<LayerElement>,
}

/// What a layer (or a beam or tuplet in it) holds.
#[derive(Debug, Clone, PartialEq)]
pub enum LayerElement {
    Note(Note),
    Rest(Rest),
    Chord(Chord),
    Beam(Beam),
    Tuplet(Tuplet),
    Clef(Clef),
    KeySig(KeySigElement),
    MeterSig(MeterSig),
    /// A control event written inside the layer, before the event it
    /// applies to (as files written by `scores` do), rather than after the
    /// staves.
    Control(ControlElement),
}

/// `<note>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Note {
    pub xml_id: Option<String>,
    pub pname: Option<PitchName>,
    pub oct: Option<i8>,
    /// Unset on a chord's notes, which take the chord's.
    pub dur: Option<Duration>,
    pub dots: Option<u8>,
    pub accid_ges: Option<Accidental>,
    pub tie: Option<Tie>,
    pub grace: Option<Grace>,
    /// `@fermata`: shorthand for a `<fermata>` on this note.
    pub fermata: Option<Place>,
    /// `@artic`: `data.ARTICULATION` values.
    pub artic: Vec<String>,
    pub content: Vec<NoteElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteElement {
    Artic(Artic),
    /// A control event nested in the note.
    Control(ControlElement),
}

/// `<rest>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rest {
    pub xml_id: Option<String>,
    pub dur: Option<Duration>,
    pub dots: Option<u8>,
    pub fermata: Option<Place>,
    /// Control events nested in the rest.
    pub content: Vec<ControlElement>,
}

/// `<chord>`: notes sounding together, sharing the chord's duration.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chord {
    pub xml_id: Option<String>,
    pub dur: Option<Duration>,
    pub dots: Option<u8>,
    pub grace: Option<Grace>,
    pub fermata: Option<Place>,
    pub artic: Vec<String>,
    pub content: Vec<ChordElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChordElement {
    Note(Note),
    Artic(Artic),
    Control(ControlElement),
}

/// `<artic>`: articulations, as `data.ARTICULATION` values (`acc`,
/// `stacc`, ...).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Artic {
    pub artic: Vec<String>,
}

/// `<clef>`: a clef change.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Clef {
    pub shape: Option<ClefShape>,
    pub line: Option<i8>,
    pub dis: Option<u8>,
    pub dis_place: Option<Place>,
}

/// `<keySig>`: a key signature change.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KeySigElement {
    pub sig: Option<KeySig>,
    pub mode: Option<Mode>,
}

/// `<meterSig>`: a time signature change.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeterSig {
    pub count: Option<u8>,
    pub unit: Option<u8>,
}

/// `<dynam>`: a dynamic marking, as text (`mf`, `sfz`, ...).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dynam {
    pub control: att::ControlEvent,
    pub text: String,
}

/// `<dir>`: a textual direction.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dir {
    pub control: att::ControlEvent,
    pub text: String,
}

/// `<ornam>`: an ornament with no element of its own. MEI uses it for
/// unnamed ornaments; files written by `scores` name the ornament in
/// `@type` (with `@count` for a tremolo's strokes).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ornam {
    pub control: att::ControlEvent,
    pub r#type: Option<String>,
    pub count: Option<u8>,
}

/// A control event: a marking tied to an event, either by its attributes
/// (`@startid`, `@tstamp`) or by where it's written.
#[derive(Debug, Clone, PartialEq)]
pub enum ControlElement {
    Harm(Harm),
    Dynam(Dynam),
    Hairpin(Hairpin),
    Dir(Dir),
    RepeatMark(RepeatMark),
    Fermata(Fermata),
    Trill(Trill),
    Mordent(Mordent),
    Turn(Turn),
    Ornam(Ornam),
    /// `<navigation type="...">`, an element of its own that older
    /// versions of `scores` wrote for D.C./D.S./Coda/Segno/Fine before
    /// `<repeatMark>`; not MEI.
    Navigation(String),
}
