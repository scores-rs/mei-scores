//! The document model to MEI XML, indented by two spaces.

use crate::Mei;
use crate::att;
use crate::cmn::{Measure, MeasureElement};
use crate::error::MeiError;
use crate::header::MeiHead;
use crate::shared::{
    Artic, Chord, ChordElement, Clef, ControlElement, KeySigElement, Layer, LayerElement, Mdiv,
    MeterSig, Note, NoteElement, Rest, Score, ScoreDef, Section, SectionElement, Staff, StaffDef,
    StaffElement, StaffGrp, StaffGrpElement,
};
use quick_xml::events::{BytesDecl, BytesText, Event};
use quick_xml::writer::Writer;
use std::io;

type W = Writer<Vec<u8>>;

/// An element's attributes, in the order they're written; unset ones are
/// left out.
#[derive(Default)]
struct Attrs(Vec<(&'static str, String)>);

impl Attrs {
    fn add(mut self, name: &'static str, value: Option<impl ToString>) -> Self {
        if let Some(value) = value {
            self.0.push((name, value.to_string()));
        }
        self
    }

    fn control(self, control: &att::ControlEvent) -> Self {
        let staff = (!control.staff.is_empty()).then(|| {
            control
                .staff
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        });
        self.add("staff", staff)
            .add("tstamp", control.tstamp)
            .add("startid", control.startid.as_ref())
    }

    fn artic(self, artic: &[String]) -> Self {
        self.add("artic", (!artic.is_empty()).then(|| artic.join(" ")))
    }
}

fn element<'a>(
    writer: &'a mut W,
    name: &'a str,
    attrs: &'a Attrs,
) -> quick_xml::writer::ElementWriter<'a, Vec<u8>> {
    writer
        .create_element(name)
        .with_attributes(attrs.0.iter().map(|(k, v)| (*k, v.as_str())))
}

/// An element that always has a start and an end tag.
fn container(
    writer: &mut W,
    name: &str,
    attrs: Attrs,
    content: impl FnOnce(&mut W) -> io::Result<()>,
) -> io::Result<()> {
    element(writer, name, &attrs).write_inner_content(content)?;
    Ok(())
}

/// An element that is self-closing when it has nothing inside.
fn leaf(
    writer: &mut W,
    name: &str,
    attrs: Attrs,
    empty: bool,
    content: impl FnOnce(&mut W) -> io::Result<()>,
) -> io::Result<()> {
    let el = element(writer, name, &attrs);
    if empty {
        el.write_empty()?;
    } else {
        el.write_inner_content(content)?;
    }
    Ok(())
}

/// An element holding text, self-closing when the text is empty.
fn text(writer: &mut W, name: &str, attrs: Attrs, text: &str) -> io::Result<()> {
    let el = element(writer, name, &attrs);
    if text.is_empty() {
        el.write_empty()?;
    } else {
        el.write_text_content(BytesText::new(text))?;
    }
    Ok(())
}

pub fn write(mei: &Mei) -> Result<Vec<u8>, MeiError> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    let attrs = Attrs::default()
        .add("xmlns", Some(crate::NAMESPACE))
        .add("meiversion", mei.meiversion.as_ref());
    container(&mut writer, "mei", attrs, |w| {
        if let Some(head) = &mei.head {
            write_head(w, head)?;
        }
        container(w, "music", Attrs::default(), |w| {
            container(w, "body", Attrs::default(), |w| {
                mei.music
                    .body
                    .mdivs
                    .iter()
                    .try_for_each(|m| write_mdiv(w, m))
            })
        })
    })?;
    Ok(writer.into_inner())
}

fn write_head(writer: &mut W, head: &MeiHead) -> io::Result<()> {
    let title_stmt = &head.file_desc.title_stmt;
    container(writer, "meiHead", Attrs::default(), |w| {
        container(w, "fileDesc", Attrs::default(), |w| {
            container(w, "titleStmt", Attrs::default(), |w| {
                if let Some(title) = &title_stmt.title {
                    w.create_element("title")
                        .write_text_content(BytesText::new(title))?;
                }
                if let Some(composer) = &title_stmt.composer {
                    w.create_element("composer")
                        .write_text_content(BytesText::new(composer))?;
                }
                Ok(())
            })
        })
    })
}

fn write_mdiv(writer: &mut W, mdiv: &Mdiv) -> io::Result<()> {
    container(writer, "mdiv", Attrs::default(), |w| {
        if let Some(score) = &mdiv.score {
            write_score(w, score)?;
        }
        mdiv.mdivs.iter().try_for_each(|m| write_mdiv(w, m))
    })
}

fn write_score(writer: &mut W, score: &Score) -> io::Result<()> {
    container(writer, "score", Attrs::default(), |w| {
        if let Some(score_def) = &score.score_def {
            write_score_def(w, score_def)?;
        }
        score.sections.iter().try_for_each(|s| write_section(w, s))
    })
}

fn write_score_def(writer: &mut W, score_def: &ScoreDef) -> io::Result<()> {
    container(writer, "scoreDef", Attrs::default(), |w| {
        write_staff_grp(w, &score_def.staff_grp)
    })
}

fn write_staff_grp(writer: &mut W, group: &StaffGrp) -> io::Result<()> {
    container(writer, "staffGrp", Attrs::default(), |w| {
        group.content.iter().try_for_each(|element| match element {
            StaffGrpElement::StaffDef(def) => write_staff_def(w, def),
            StaffGrpElement::StaffGrp(group) => write_staff_grp(w, group),
        })
    })
}

fn write_staff_def(writer: &mut W, def: &StaffDef) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("n", def.n)
        .add("lines", def.lines)
        .add("label", def.label.as_ref())
        .add("clef.shape", def.clef_shape.as_ref())
        .add("clef.line", def.clef_line)
        .add("clef.dis", def.clef_dis)
        .add("clef.dis.place", def.clef_dis_place.as_ref())
        .add("key.sig", def.key_sig)
        .add("key.mode", def.key_mode.as_ref())
        .add("meter.count", def.meter_count)
        .add("meter.unit", def.meter_unit)
        .add("trans.diat", def.trans_diat)
        .add("trans.semi", def.trans_semi);
    element(writer, "staffDef", &attrs).write_empty()?;
    Ok(())
}

fn write_section(writer: &mut W, section: &Section) -> io::Result<()> {
    container(writer, "section", Attrs::default(), |w| {
        write_section_content(w, &section.content)
    })
}

fn write_section_content(writer: &mut W, content: &[SectionElement]) -> io::Result<()> {
    content.iter().try_for_each(|element| match element {
        SectionElement::Measure(measure) => write_measure(writer, measure),
        SectionElement::Ending(ending) => {
            let attrs = Attrs::default()
                .add("n", ending.n.as_ref())
                .add("lendsym", ending.lendsym.as_ref())
                .add("type", ending.r#type.as_ref());
            container(writer, "ending", attrs, |w| {
                write_section_content(w, &ending.content)
            })
        }
        SectionElement::ScoreDef(score_def) => write_score_def(writer, score_def),
        SectionElement::Section(section) => write_section(writer, section),
    })
}

fn write_measure(writer: &mut W, measure: &Measure) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("n", measure.n)
        .add("left", measure.left.as_ref())
        .add("right", measure.right.as_ref())
        .add("repeat.count", measure.repeat_count);
    container(writer, "measure", attrs, |w| {
        measure
            .content
            .iter()
            .try_for_each(|element| match element {
                MeasureElement::Staff(staff) => write_staff(w, staff),
                MeasureElement::Control(control) => write_control(w, control),
            })
    })
}

fn write_staff(writer: &mut W, staff: &Staff) -> io::Result<()> {
    container(writer, "staff", Attrs::default().add("n", staff.n), |w| {
        staff.content.iter().try_for_each(|element| match element {
            StaffElement::Layer(layer) => write_layer(w, layer),
            StaffElement::Clef(clef) => write_clef(w, clef),
            StaffElement::KeySig(key) => write_key_sig(w, key),
            StaffElement::MeterSig(meter) => write_meter_sig(w, meter),
            StaffElement::Control(control) => write_control(w, control),
        })
    })
}

fn write_layer(writer: &mut W, layer: &Layer) -> io::Result<()> {
    container(writer, "layer", Attrs::default().add("n", layer.n), |w| {
        write_layer_content(w, &layer.content)
    })
}

fn write_layer_content(writer: &mut W, content: &[LayerElement]) -> io::Result<()> {
    content.iter().try_for_each(|element| match element {
        LayerElement::Note(note) => write_note(writer, note),
        LayerElement::Rest(rest) => write_rest(writer, rest),
        LayerElement::Chord(chord) => write_chord(writer, chord),
        LayerElement::Beam(beam) => container(writer, "beam", Attrs::default(), |w| {
            write_layer_content(w, &beam.content)
        }),
        LayerElement::Tuplet(tuplet) => {
            let attrs = Attrs::default()
                .add("num", tuplet.num)
                .add("numbase", tuplet.numbase);
            container(writer, "tuplet", attrs, |w| {
                write_layer_content(w, &tuplet.content)
            })
        }
        LayerElement::Clef(clef) => write_clef(writer, clef),
        LayerElement::KeySig(key) => write_key_sig(writer, key),
        LayerElement::MeterSig(meter) => write_meter_sig(writer, meter),
        LayerElement::Control(control) => write_control(writer, control),
    })
}

fn write_clef(writer: &mut W, clef: &Clef) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("shape", clef.shape.as_ref())
        .add("line", clef.line)
        .add("dis", clef.dis)
        .add("dis.place", clef.dis_place.as_ref());
    element(writer, "clef", &attrs).write_empty()?;
    Ok(())
}

fn write_key_sig(writer: &mut W, key: &KeySigElement) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("sig", key.sig)
        .add("mode", key.mode.as_ref());
    element(writer, "keySig", &attrs).write_empty()?;
    Ok(())
}

fn write_meter_sig(writer: &mut W, meter: &MeterSig) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("count", meter.count)
        .add("unit", meter.unit);
    element(writer, "meterSig", &attrs).write_empty()?;
    Ok(())
}

fn write_note(writer: &mut W, note: &Note) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("xml:id", note.xml_id.as_ref())
        .add("pname", note.pname.map(|p| p.as_str()))
        .add("oct", note.oct)
        .add("dur", note.dur.as_ref())
        .add("dots", note.dots)
        .add("accid.ges", note.accid_ges.as_ref())
        .add("tie", note.tie.as_ref())
        .add("grace", note.grace.as_ref())
        .add("fermata", note.fermata.as_ref())
        .artic(&note.artic);
    leaf(writer, "note", attrs, note.content.is_empty(), |w| {
        note.content.iter().try_for_each(|element| match element {
            NoteElement::Artic(artic) => write_artic(w, artic),
            NoteElement::Control(control) => write_control(w, control),
        })
    })
}

fn write_rest(writer: &mut W, rest: &Rest) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("xml:id", rest.xml_id.as_ref())
        .add("dur", rest.dur.as_ref())
        .add("dots", rest.dots)
        .add("fermata", rest.fermata.as_ref());
    leaf(writer, "rest", attrs, rest.content.is_empty(), |w| {
        rest.content.iter().try_for_each(|c| write_control(w, c))
    })
}

fn write_chord(writer: &mut W, chord: &Chord) -> io::Result<()> {
    let attrs = Attrs::default()
        .add("xml:id", chord.xml_id.as_ref())
        .add("dur", chord.dur.as_ref())
        .add("dots", chord.dots)
        .add("grace", chord.grace.as_ref())
        .add("fermata", chord.fermata.as_ref())
        .artic(&chord.artic);
    container(writer, "chord", attrs, |w| {
        chord.content.iter().try_for_each(|element| match element {
            ChordElement::Note(note) => write_note(w, note),
            ChordElement::Artic(artic) => write_artic(w, artic),
            ChordElement::Control(control) => write_control(w, control),
        })
    })
}

fn write_artic(writer: &mut W, artic: &Artic) -> io::Result<()> {
    element(writer, "artic", &Attrs::default().artic(&artic.artic)).write_empty()?;
    Ok(())
}

fn write_control(writer: &mut W, control: &ControlElement) -> io::Result<()> {
    let empty = |writer: &mut W, name: &str, attrs: Attrs| -> io::Result<()> {
        element(writer, name, &attrs).write_empty()?;
        Ok(())
    };
    match control {
        ControlElement::Harm(harm) => text(
            writer,
            "harm",
            Attrs::default().control(&harm.control),
            &harm.text,
        ),
        ControlElement::Dynam(dynam) => text(
            writer,
            "dynam",
            Attrs::default().control(&dynam.control),
            &dynam.text,
        ),
        ControlElement::Dir(dir) => text(
            writer,
            "dir",
            Attrs::default().control(&dir.control),
            &dir.text,
        ),
        ControlElement::RepeatMark(mark) => text(
            writer,
            "repeatMark",
            Attrs::default()
                .add("func", mark.func.as_ref())
                .control(&mark.control),
            &mark.text,
        ),
        ControlElement::Hairpin(hairpin) => empty(
            writer,
            "hairpin",
            Attrs::default()
                .add("form", hairpin.form.as_ref())
                .control(&hairpin.control)
                .add("endid", hairpin.endid.as_ref())
                .add("tstamp2", hairpin.tstamp2.as_ref()),
        ),
        ControlElement::Fermata(fermata) => empty(
            writer,
            "fermata",
            Attrs::default()
                .add("place", fermata.place.as_ref())
                .control(&fermata.control),
        ),
        ControlElement::Trill(trill) => {
            empty(writer, "trill", Attrs::default().control(&trill.control))
        }
        ControlElement::Mordent(mordent) => empty(
            writer,
            "mordent",
            Attrs::default()
                .add("form", mordent.form.as_ref())
                .control(&mordent.control),
        ),
        ControlElement::Turn(turn) => empty(
            writer,
            "turn",
            Attrs::default()
                .add("form", turn.form.as_ref())
                .control(&turn.control),
        ),
        ControlElement::Ornam(ornam) => empty(
            writer,
            "ornam",
            Attrs::default()
                .add("type", ornam.r#type.as_ref())
                .add("count", ornam.count)
                .control(&ornam.control),
        ),
        ControlElement::Navigation(kind) => empty(
            writer,
            "navigation",
            Attrs::default().add("type", Some(kind)),
        ),
    }
}
