//! Reading MEI into the document model and writing it back.

use mei_scores::Mei;
use mei_scores::cmn::MeasureElement;
use mei_scores::data::{
    Accidental, BarRendition, ClefShape, Duration, HairpinForm, KeySig, Mode, OrnamentForm,
    PitchName, Place, RepeatMarkFunc, Tie,
};
use mei_scores::shared::{
    ChordElement, ControlElement, LayerElement, NoteElement, SectionElement, StaffElement,
    StaffGrpElement,
};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei" meiversion="5.0">
  <meiHead><fileDesc><titleStmt>
    <title>Tom &amp; <rend>Jerry</rend></title><composer>Anon.</composer>
  </titleStmt></fileDesc></meiHead>
  <music><body><mdiv><score>
    <scoreDef><staffGrp>
      <staffDef n="1" lines="5" label="Clarinet" clef.shape="G" clef.line="2" key.sig="3f" key.mode="minor" meter.count="6" meter.unit="8" trans.diat="-1" trans.semi="-2"/>
      <staffGrp><staffDef n="2" clef.shape="F" clef.line="4" clef.dis="8" clef.dis.place="below"/></staffGrp>
    </staffGrp></scoreDef>
    <section>
      <measure n="1" right="rptboth">
        <staff n="1"><layer n="1">
          <note xml:id="n1" pname="c" oct="4" dur="4" accid.ges="s" tie="i" artic="acc stacc"/>
          <beam>
            <chord xml:id="c1" dur="8" dots="1" fermata="above">
              <artic artic="ten"/>
              <note pname="e" oct="4"/><note pname="G" oct="4"/>
            </chord>
          </beam>
          <app><rdg><rest dur="breve"><fermata/></rest></rdg></app>
          <tuplet num="3" numbase="2"><note pname="d" oct="5" dur="8"><mordent form="upper"/></note></tuplet>
        </layer></staff>
        <harm staff="1" startid="#n1">Dm7</harm>
        <hairpin staff="1 2" form="cres" tstamp="1.5" tstamp2="0m+3"/>
        <repeatMark func="segno" staff="1" tstamp="1"/>
      </measure>
      <ending n="1,2" lendsym="none">
        <measure n="2" right="heavyLight"><staff n="1"><layer n="1"><clef shape="C" line="4"/><rest dur="1"/></layer></staff></measure>
      </ending>
    </section>
  </score></mdiv></body></music>
</mei>"##;

#[test]
fn reads_the_document_structure() {
    let mei = Mei::from_bytes(DOC.as_bytes()).unwrap();
    assert_eq!(mei.meiversion.as_deref(), Some("5.0"));
    let title_stmt = &mei.head.as_ref().unwrap().file_desc.title_stmt;
    // Entity references and nested elements' text are part of the text.
    assert_eq!(title_stmt.title.as_deref(), Some("Tom&Jerry"));
    assert_eq!(title_stmt.composer.as_deref(), Some("Anon."));

    let score = mei.music.body.mdivs[0].score.as_ref().unwrap();
    let staff_grp = &score.score_def.as_ref().unwrap().staff_grp;
    assert!(matches!(staff_grp.content[1], StaffGrpElement::StaffGrp(_)));
    let defs = staff_grp.staff_defs();
    assert_eq!(defs.len(), 2);
    assert_eq!(defs[0].label.as_deref(), Some("Clarinet"));
    assert_eq!(
        (defs[0].trans_diat, defs[0].trans_semi),
        (Some(-1), Some(-2))
    );
    assert_eq!((defs[1].trans_diat, defs[1].trans_semi), (None, None));
    assert_eq!(defs[0].key_sig, Some(KeySig { fifths: -3 }));
    assert_eq!(defs[0].key_mode, Some(Mode::Minor));
    assert_eq!(
        (defs[0].meter_count, defs[0].meter_unit),
        (Some(6), Some(8))
    );
    assert_eq!(defs[1].clef_shape, Some(ClefShape::F));
    assert_eq!(defs[1].clef_dis, Some(8));
    assert_eq!(defs[1].clef_dis_place, Some(Place::Below));

    let content = &score.sections[0].content;
    let SectionElement::Measure(measure) = &content[0] else {
        panic!()
    };
    assert_eq!(measure.n, Some(1));
    assert_eq!(measure.right, Some(BarRendition::RptBoth));
    let MeasureElement::Staff(staff) = &measure.content[0] else {
        panic!()
    };
    let StaffElement::Layer(layer) = &staff.content[0] else {
        panic!()
    };
    let LayerElement::Note(note) = &layer.content[0] else {
        panic!()
    };
    assert_eq!(note.xml_id.as_deref(), Some("n1"));
    assert_eq!(note.pname, Some(PitchName::C));
    assert_eq!(note.dur, Some(Duration::Fraction(4)));
    assert_eq!(note.accid_ges, Some(Accidental::Sharp));
    assert_eq!(note.tie, Some(Tie::Initial));
    assert_eq!(note.artic, ["acc", "stacc"]);

    let LayerElement::Beam(beam) = &layer.content[1] else {
        panic!()
    };
    let LayerElement::Chord(chord) = &beam.content[0] else {
        panic!()
    };
    assert_eq!(chord.dots, Some(1));
    assert_eq!(chord.fermata, Some(Place::Above));
    assert!(matches!(&chord.content[0], ChordElement::Artic(a) if a.artic == ["ten"]));
    let ChordElement::Note(upper) = &chord.content[2] else {
        panic!()
    };
    assert_eq!(upper.pname, Some(PitchName::G));
    assert_eq!(upper.dur, None);

    // Unmodeled elements are looked through.
    let LayerElement::Rest(rest) = &layer.content[2] else {
        panic!()
    };
    assert_eq!(rest.dur, Some(Duration::Breve));
    assert!(matches!(rest.content[0], ControlElement::Fermata(_)));

    let LayerElement::Tuplet(tuplet) = &layer.content[3] else {
        panic!()
    };
    assert_eq!((tuplet.num, tuplet.numbase), (Some(3), Some(2)));
    let LayerElement::Note(ornamented) = &tuplet.content[0] else {
        panic!()
    };
    assert!(matches!(
        &ornamented.content[0],
        NoteElement::Control(ControlElement::Mordent(m)) if m.form == Some(OrnamentForm::Upper)
    ));

    let MeasureElement::Control(ControlElement::Harm(harm)) = &measure.content[1] else {
        panic!()
    };
    assert_eq!(harm.text, "Dm7");
    assert_eq!(harm.control.start_id(), Some("n1"));
    let MeasureElement::Control(ControlElement::Hairpin(hairpin)) = &measure.content[2] else {
        panic!()
    };
    assert_eq!(hairpin.form, Some(HairpinForm::Cres));
    assert_eq!(hairpin.control.staff, [1, 2]);
    assert_eq!(hairpin.control.tstamp, Some(1.5));
    assert_eq!(hairpin.tstamp2.as_deref(), Some("0m+3"));
    let MeasureElement::Control(ControlElement::RepeatMark(mark)) = &measure.content[3] else {
        panic!()
    };
    assert_eq!(mark.func, Some(RepeatMarkFunc::Segno));

    let SectionElement::Ending(ending) = &content[1] else {
        panic!()
    };
    assert_eq!(ending.n.as_deref(), Some("1,2"));
    assert_eq!(ending.lendsym.as_deref(), Some("none"));
    let SectionElement::Measure(second) = &ending.content[0] else {
        panic!()
    };
    // A value outside data.BARRENDITION is kept.
    assert_eq!(second.right, Some(BarRendition::Other("heavyLight".into())));
}

#[test]
fn written_documents_read_back_the_same() {
    let mei = Mei::from_bytes(DOC.as_bytes()).unwrap();
    let written = mei.to_bytes().unwrap();
    assert_eq!(Mei::from_bytes(&written).unwrap(), mei);
}

#[test]
fn writes_indented_xml() {
    let mei = Mei::from_bytes(
        br#"<mei meiversion="5.0"><music><body><mdiv><score><section>
             <measure n="1" left="rptstart"><staff n="1"><layer n="1">
               <dynam>mf</dynam><note pname="c" oct="4" dur="4" dots="0"/>
             </layer></staff><repeatMark func="fine" staff="1" tstamp="5">Fine</repeatMark></measure>
           </section></score></mdiv></body></music></mei>"#,
    )
    .unwrap();
    let text = String::from_utf8(mei.to_bytes().unwrap()).unwrap();
    assert_eq!(
        text,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei" meiversion="5.0">
  <music>
    <body>
      <mdiv>
        <score>
          <section>
            <measure n="1" left="rptstart">
              <staff n="1">
                <layer n="1">
                  <dynam>mf</dynam>
                  <note pname="c" oct="4" dur="4" dots="0"/>
                </layer>
              </staff>
              <repeatMark func="fine" staff="1" tstamp="5">Fine</repeatMark>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#
    );
}

#[test]
fn rejects_malformed_input() {
    assert!(Mei::from_bytes(b"<mei><music></mei>").is_err());
    assert!(Mei::from_bytes(&[0xff, 0xfe]).is_err());
}
