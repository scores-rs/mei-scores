//! MEI XML to the document model.
//!
//! The XML is first read into a plain element tree, then each element is
//! turned into its typed record. Readers are lenient: an element that
//! isn't modeled in some place is looked through, so its modeled
//! descendants still count (a `<note>` inside an `<app>`/`<rdg>`, a
//! `<layer>` inside a `<choice>`).

use crate::Mei;
use crate::att;
use crate::cmn::{Beam, Fermata, Hairpin, Measure, MeasureElement, RepeatMark, Tuplet};
use crate::data::{
    Accidental, BarRendition, ClefShape, Duration, Grace, HairpinForm, KeySig, Mode, OrnamentForm,
    PitchName, Place, RepeatMarkFunc, Tie,
};
use crate::error::MeiError;
use crate::harmony::Harm;
use crate::header::{FileDesc, MeiHead, TitleStmt};
use crate::ornaments::{Mordent, Trill, Turn};
use crate::shared::{
    Artic, Chord, ChordElement, Clef, ControlElement, Dir, Dynam, Ending, KeySigElement, Layer,
    LayerElement, Mdiv, MeterSig, Note, NoteElement, Ornam, Rest, Score, ScoreDef, Section,
    SectionElement, Staff, StaffDef, StaffElement, StaffGrp, StaffGrpElement,
};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::str::FromStr;

/// An XML element, before it's given a type.
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Node>,
}

enum Node {
    Element(Element),
    Text(String),
}

fn start(e: &BytesStart) -> Element {
    let attributes = e
        .attributes()
        .flatten()
        .filter_map(|a| {
            let key = a.key.as_ref().to_string();
            let value = a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()?
                .into_owned();
            Some((key, value))
        })
        .collect();
    Element {
        name: e.local_name().as_ref().to_string(),
        attributes,
        children: Vec::new(),
    }
}

/// Reads the XML into an element tree (rooted in a nameless element holding
/// the document's top-level elements).
fn parse_tree(text: &str) -> Result<Element, MeiError> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(true);
    let xml = |e: quick_xml::Error| MeiError::Xml(e.to_string());
    let mut stack = vec![Element {
        name: String::new(),
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    loop {
        match reader.read_event().map_err(xml)? {
            Event::Eof => break,
            Event::Start(e) => stack.push(start(&e)),
            Event::Empty(e) => {
                let element = start(&e);
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(Node::Element(element));
                }
            }
            Event::End(_) => {
                if stack.len() > 1 {
                    let element = stack.pop().unwrap();
                    stack
                        .last_mut()
                        .unwrap()
                        .children
                        .push(Node::Element(element));
                }
            }
            Event::Text(text) => {
                let raw = text.into_inner();
                if let Ok(unescaped) = quick_xml::escape::unescape(&raw)
                    && let Some(parent) = stack.last_mut()
                {
                    parent.children.push(Node::Text(unescaped.into_owned()));
                }
            }
            // `&amp;`, `&#233;`, ...: read as the text they stand for.
            Event::GeneralRef(reference) => {
                let resolved = match reference.resolve_char_ref() {
                    Ok(Some(c)) => Some(c.to_string()),
                    _ => {
                        quick_xml::escape::resolve_predefined_entity(&reference).map(str::to_string)
                    }
                };
                if let (Some(text), Some(parent)) = (resolved, stack.last_mut()) {
                    parent.children.push(Node::Text(text));
                }
            }
            _ => {}
        }
    }
    while stack.len() > 1 {
        let element = stack.pop().unwrap();
        stack
            .last_mut()
            .unwrap()
            .children
            .push(Node::Element(element));
    }
    Ok(stack.pop().unwrap())
}

impl Element {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn string(&self, name: &str) -> Option<String> {
        self.attr(name).map(str::to_string)
    }

    fn parse<T: FromStr>(&self, name: &str) -> Option<T> {
        self.attr(name)?.parse().ok()
    }

    fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|node| match node {
            Node::Element(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    /// Children named in `known`, looking through any other element.
    fn find<'a>(&'a self, known: &[&str], out: &mut Vec<&'a Element>) {
        for child in self.elements() {
            if known.contains(&child.name.as_str()) {
                out.push(child);
            } else {
                child.find(known, out);
            }
        }
    }

    fn found(&self, known: &[&str]) -> Vec<&Element> {
        let mut out = Vec::new();
        self.find(known, &mut out);
        out
    }

    /// All text inside the element, nested elements' included.
    fn text(&self) -> String {
        let mut out = String::new();
        for node in &self.children {
            match node {
                Node::Text(text) => out.push_str(text),
                Node::Element(e) => out.push_str(&e.text()),
            }
        }
        out
    }

    /// `@artic`: a space-separated list.
    fn artic(&self) -> Vec<String> {
        self.attr("artic")
            .map(|codes| codes.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn control(&self) -> att::ControlEvent {
        att::ControlEvent {
            startid: self.string("startid"),
            tstamp: self.attr("tstamp").and_then(|t| t.trim().parse().ok()),
            staff: self
                .attr("staff")
                .map(|s| {
                    s.split_whitespace()
                        .filter_map(|n| n.parse().ok())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

const CONTROLS: &[&str] = &[
    "harm",
    "dynam",
    "hairpin",
    "dir",
    "repeatMark",
    "fermata",
    "trill",
    "mordent",
    "turn",
    "ornam",
    "navigation",
];

fn with_controls(names: &[&'static str]) -> Vec<&'static str> {
    names.iter().chain(CONTROLS).copied().collect()
}

pub fn read(data: &[u8]) -> Result<Mei, MeiError> {
    let text = std::str::from_utf8(data)?;
    let root = parse_tree(text)?;
    let mut mei = Mei::default();
    for element in root.found(&["mei", "meiHead", "music"]) {
        match element.name.as_str() {
            "mei" => {
                mei.meiversion = element.string("meiversion");
                for part in element.found(&["meiHead", "music"]) {
                    read_top(&mut mei, part);
                }
            }
            _ => read_top(&mut mei, element),
        }
    }
    Ok(mei)
}

fn read_top(mei: &mut Mei, element: &Element) {
    match element.name.as_str() {
        "meiHead" => mei.head = Some(head(element)),
        _ => mei.music.body.mdivs.extend(mdivs(element)),
    }
}

fn head(element: &Element) -> MeiHead {
    let mut title_stmt = TitleStmt::default();
    for e in element.found(&["title", "composer"]) {
        let slot = if e.name == "title" {
            &mut title_stmt.title
        } else {
            &mut title_stmt.composer
        };
        slot.get_or_insert_with(String::new).push_str(&e.text());
    }
    MeiHead {
        file_desc: FileDesc { title_stmt },
    }
}

/// The divisions inside `<music>`/`<body>`; a `<score>` outside any
/// `<mdiv>` gets one of its own.
fn mdivs(element: &Element) -> Vec<Mdiv> {
    element
        .found(&["mdiv", "score"])
        .into_iter()
        .map(|e| match e.name.as_str() {
            "mdiv" => mdiv(e),
            _ => Mdiv {
                score: Some(score(e)),
                mdivs: Vec::new(),
            },
        })
        .collect()
}

fn mdiv(element: &Element) -> Mdiv {
    let mut out = Mdiv::default();
    for child in element.found(&["mdiv", "score"]) {
        match child.name.as_str() {
            "score" => {
                out.score.get_or_insert_with(|| score(child));
            }
            _ => out.mdivs.push(mdiv(child)),
        }
    }
    out
}

fn score(element: &Element) -> Score {
    let mut out = Score::default();
    // Whether the last section collects measures written outside any.
    let mut loose = false;
    for e in element.found(&["scoreDef", "section", "measure", "ending"]) {
        match e.name.as_str() {
            "scoreDef" if out.score_def.is_none() && out.sections.is_empty() => {
                out.score_def = Some(score_def(e));
            }
            "section" => {
                out.sections.push(section(e));
                loose = false;
            }
            _ => {
                let content = section_elements_of(&[e]);
                match out.sections.last_mut() {
                    Some(last) if loose => last.content.extend(content),
                    _ => {
                        out.sections.push(Section { content });
                        loose = true;
                    }
                }
            }
        }
    }
    out
}

/// `<scoreDef>`'s `<staffGrp>`; `<staffDef>`s outside one are gathered
/// into one.
fn score_def(element: &Element) -> ScoreDef {
    let mut content = staff_grp_content(element);
    let staff_grp = match content.as_slice() {
        [StaffGrpElement::StaffGrp(_)] => match content.pop() {
            Some(StaffGrpElement::StaffGrp(group)) => group,
            _ => unreachable!(),
        },
        _ => StaffGrp { content },
    };
    ScoreDef { staff_grp }
}

fn staff_grp_content(element: &Element) -> Vec<StaffGrpElement> {
    element
        .found(&["staffGrp", "staffDef"])
        .into_iter()
        .map(|e| match e.name.as_str() {
            "staffGrp" => StaffGrpElement::StaffGrp(StaffGrp {
                content: staff_grp_content(e),
            }),
            _ => StaffGrpElement::StaffDef(staff_def(e)),
        })
        .collect()
}

fn staff_def(e: &Element) -> StaffDef {
    StaffDef {
        n: e.parse("n"),
        lines: e.parse("lines"),
        label: e.string("label"),
        clef_shape: e.attr("clef.shape").map(ClefShape::parse),
        clef_line: e.parse("clef.line"),
        clef_dis: e.parse("clef.dis"),
        clef_dis_place: e.attr("clef.dis.place").map(Place::parse),
        key_sig: e.attr("key.sig").map(KeySig::parse),
        key_mode: e.attr("key.mode").map(Mode::parse),
        meter_count: e.parse("meter.count"),
        meter_unit: e.parse("meter.unit"),
        trans_diat: e.parse("trans.diat"),
        trans_semi: e.parse("trans.semi"),
    }
}

const SECTION: &[&str] = &["measure", "ending", "scoreDef", "section"];

fn section(element: &Element) -> Section {
    Section {
        content: section_elements_of(&element.found(SECTION)),
    }
}

fn section_elements_of(elements: &[&Element]) -> Vec<SectionElement> {
    elements
        .iter()
        .map(|e| match e.name.as_str() {
            "measure" => SectionElement::Measure(measure(e)),
            "ending" => SectionElement::Ending(Ending {
                n: e.string("n"),
                lendsym: e.string("lendsym"),
                r#type: e.string("type"),
                content: section_elements_of(&e.found(SECTION)),
            }),
            "scoreDef" => SectionElement::ScoreDef(score_def(e)),
            _ => SectionElement::Section(section(e)),
        })
        .collect()
}

fn measure(e: &Element) -> Measure {
    let content = e
        .found(&with_controls(&["staff"]))
        .into_iter()
        .map(|child| match child.name.as_str() {
            "staff" => MeasureElement::Staff(staff(child)),
            _ => MeasureElement::Control(control(child)),
        })
        .collect();
    Measure {
        n: e.parse("n"),
        left: e.attr("left").map(BarRendition::parse),
        right: e.attr("right").map(BarRendition::parse),
        repeat_count: e.parse("repeat.count"),
        content,
    }
}

fn staff(e: &Element) -> Staff {
    let content = e
        .found(&with_controls(&["layer", "clef", "keySig", "meterSig"]))
        .into_iter()
        .map(|child| match child.name.as_str() {
            "layer" => StaffElement::Layer(Layer {
                n: child.parse("n"),
                content: layer_content(child),
            }),
            "clef" => StaffElement::Clef(clef(child)),
            "keySig" => StaffElement::KeySig(key_sig(child)),
            "meterSig" => StaffElement::MeterSig(meter_sig(child)),
            _ => StaffElement::Control(control(child)),
        })
        .collect();
    Staff {
        n: e.parse("n"),
        content,
    }
}

const LAYER: &[&str] = &[
    "note", "rest", "chord", "beam", "tuplet", "clef", "keySig", "meterSig",
];

fn layer_content(e: &Element) -> Vec<LayerElement> {
    e.found(&with_controls(LAYER))
        .into_iter()
        .map(|child| match child.name.as_str() {
            "note" => LayerElement::Note(note(child)),
            "rest" => LayerElement::Rest(Rest {
                xml_id: child.string("xml:id"),
                dur: child.attr("dur").map(Duration::parse),
                dots: child.parse("dots"),
                fermata: child.attr("fermata").map(Place::parse),
                content: child.found(CONTROLS).into_iter().map(control).collect(),
            }),
            "chord" => LayerElement::Chord(chord(child)),
            "beam" => LayerElement::Beam(Beam {
                content: layer_content(child),
            }),
            "tuplet" => LayerElement::Tuplet(Tuplet {
                num: child.parse("num"),
                numbase: child.parse("numbase"),
                content: layer_content(child),
            }),
            "clef" => LayerElement::Clef(clef(child)),
            "keySig" => LayerElement::KeySig(key_sig(child)),
            "meterSig" => LayerElement::MeterSig(meter_sig(child)),
            _ => LayerElement::Control(control(child)),
        })
        .collect()
}

fn note(e: &Element) -> Note {
    let content = e
        .found(&with_controls(&["artic"]))
        .into_iter()
        .map(|child| match child.name.as_str() {
            "artic" => NoteElement::Artic(Artic {
                artic: child.artic(),
            }),
            _ => NoteElement::Control(control(child)),
        })
        .collect();
    Note {
        xml_id: e.string("xml:id"),
        pname: e.attr("pname").and_then(PitchName::parse),
        oct: e.parse("oct"),
        dur: e.attr("dur").map(Duration::parse),
        dots: e.parse("dots"),
        accid_ges: e.attr("accid.ges").map(Accidental::parse),
        tie: e.attr("tie").map(Tie::parse),
        grace: e.attr("grace").map(Grace::parse),
        fermata: e.attr("fermata").map(Place::parse),
        artic: e.artic(),
        content,
    }
}

fn chord(e: &Element) -> Chord {
    let content = e
        .found(&with_controls(&["note", "artic"]))
        .into_iter()
        .map(|child| match child.name.as_str() {
            "note" => ChordElement::Note(note(child)),
            "artic" => ChordElement::Artic(Artic {
                artic: child.artic(),
            }),
            _ => ChordElement::Control(control(child)),
        })
        .collect();
    Chord {
        xml_id: e.string("xml:id"),
        dur: e.attr("dur").map(Duration::parse),
        dots: e.parse("dots"),
        grace: e.attr("grace").map(Grace::parse),
        fermata: e.attr("fermata").map(Place::parse),
        artic: e.artic(),
        content,
    }
}

fn clef(e: &Element) -> Clef {
    Clef {
        shape: e.attr("shape").map(ClefShape::parse),
        line: e.parse("line"),
        dis: e.parse("dis"),
        dis_place: e.attr("dis.place").map(Place::parse),
    }
}

fn key_sig(e: &Element) -> KeySigElement {
    KeySigElement {
        sig: e.attr("sig").map(KeySig::parse),
        mode: e.attr("mode").map(Mode::parse),
    }
}

fn meter_sig(e: &Element) -> MeterSig {
    MeterSig {
        count: e.parse("count"),
        unit: e.parse("unit"),
    }
}

fn control(e: &Element) -> ControlElement {
    let control = e.control();
    match e.name.as_str() {
        "harm" => ControlElement::Harm(Harm {
            control,
            text: e.text(),
        }),
        "dynam" => ControlElement::Dynam(Dynam {
            control,
            text: e.text(),
        }),
        "dir" => ControlElement::Dir(Dir {
            control,
            text: e.text(),
        }),
        "repeatMark" => ControlElement::RepeatMark(RepeatMark {
            control,
            func: e.attr("func").map(RepeatMarkFunc::parse),
            text: e.text(),
        }),
        "hairpin" => ControlElement::Hairpin(Hairpin {
            control,
            form: e.attr("form").map(HairpinForm::parse),
            endid: e.string("endid"),
            tstamp2: e.string("tstamp2"),
        }),
        "fermata" => ControlElement::Fermata(Fermata {
            control,
            place: e.attr("place").map(Place::parse),
        }),
        "trill" => ControlElement::Trill(Trill { control }),
        "mordent" => ControlElement::Mordent(Mordent {
            control,
            form: e.attr("form").map(OrnamentForm::parse),
        }),
        "turn" => ControlElement::Turn(Turn {
            control,
            form: e.attr("form").map(OrnamentForm::parse),
        }),
        "ornam" => ControlElement::Ornam(Ornam {
            control,
            r#type: e.string("type"),
            count: e.parse("count"),
        }),
        _ => ControlElement::Navigation(e.string("type").unwrap_or_default()),
    }
}
