//! MEI's data types (`data.*` in the guidelines): the vocabularies attribute
//! values are drawn from. Each type parses the attribute's text and writes
//! it back; values outside the vocabulary are kept as `Other` so nothing a
//! file says is lost.

use std::fmt;

macro_rules! vocabulary {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident = $text:literal,)* }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $($(#[$vmeta])* $variant,)*
            /// A value outside the vocabulary, as written.
            Other(String),
        }

        impl $name {
            /// Parses an attribute value.
            pub fn parse(value: &str) -> Self {
                match value {
                    $($text => Self::$variant,)*
                    other => Self::Other(other.to_string()),
                }
            }

            /// The attribute value.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $text,)*
                    Self::Other(other) => other,
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

/// `data.DURATION`: a note value as the denominator of a fraction of a
/// whole note (`4` is a quarter), or one of the longer values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Duration {
    Long,
    Breve,
    /// `1`, `2`, `4`, `8`, ... (`2048` at most).
    Fraction(u32),
    /// A value outside the vocabulary, as written.
    Other(String),
}

impl Duration {
    pub fn parse(value: &str) -> Self {
        match value {
            "long" => Self::Long,
            "breve" => Self::Breve,
            other => other
                .parse()
                .map(Self::Fraction)
                .unwrap_or_else(|_| Self::Other(other.to_string())),
        }
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Long => f.write_str("long"),
            Self::Breve => f.write_str("breve"),
            Self::Fraction(n) => write!(f, "{n}"),
            Self::Other(other) => f.write_str(other),
        }
    }
}

/// `data.PITCHNAME`: a pitch's letter name, `c` to `b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchName {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl PitchName {
    /// Parses a pitch name, ignoring case.
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value.to_ascii_lowercase().as_str() {
            "c" => Self::C,
            "d" => Self::D,
            "e" => Self::E,
            "f" => Self::F,
            "g" => Self::G,
            "a" => Self::A,
            "b" => Self::B,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::C => "c",
            Self::D => "d",
            Self::E => "e",
            Self::F => "f",
            Self::G => "g",
            Self::A => "a",
            Self::B => "b",
        }
    }
}

vocabulary! {
    /// `data.ACCIDENTAL.GESTURAL`: an accidental that sounds but isn't
    /// necessarily written (`@accid.ges`).
    Accidental {
        Sharp = "s",
        Flat = "f",
        DoubleSharp = "ss",
        DoubleFlat = "ff",
        Natural = "n",
    }
}

vocabulary! {
    /// `data.TIE`: where a note sits in a tie.
    Tie {
        Initial = "i",
        Medial = "m",
        Terminal = "t",
    }
}

vocabulary! {
    /// `data.GRACE`: whether a grace note takes time from the note before
    /// (`acc`, an acciaccatura) or after it (`unacc`, an appoggiatura).
    Grace {
        Acc = "acc",
        Unacc = "unacc",
        Unknown = "unknown",
    }
}

vocabulary! {
    /// `data.STAFFREL.basic`: above or below the staff (fermatas, clef
    /// octave marks).
    Place {
        Above = "above",
        Below = "below",
    }
}

vocabulary! {
    /// `data.CLEFSHAPE`.
    ClefShape {
        G = "G",
        GG = "GG",
        F = "F",
        C = "C",
        Perc = "perc",
        Tab = "TAB",
    }
}

vocabulary! {
    /// `data.MODE`: a key's mode.
    Mode {
        Major = "major",
        Minor = "minor",
        Dorian = "dorian",
        Phrygian = "phrygian",
        Lydian = "lydian",
        Mixolydian = "mixolydian",
        Aeolian = "aeolian",
        Locrian = "locrian",
    }
}

impl Mode {
    /// Whether this is minor, ignoring case (`Minor` from loosely written
    /// files included).
    pub fn is_minor(&self) -> bool {
        self.as_str().eq_ignore_ascii_case("minor")
    }
}

vocabulary! {
    /// `data.BARRENDITION`: a barline's look (`@left`/`@right` on
    /// `<measure>`). MEI has no heavy-light, tick or short barline; files
    /// that need them use their own values (`Other`).
    BarRendition {
        Dashed = "dashed",
        Dotted = "dotted",
        Dbl = "dbl",
        DblDashed = "dbldashed",
        DblDotted = "dbldotted",
        DblHeavy = "dblheavy",
        DblSegno = "dblsegno",
        End = "end",
        Heavy = "heavy",
        Invis = "invis",
        RptStart = "rptstart",
        RptBoth = "rptboth",
        RptEnd = "rptend",
        Segno = "segno",
        Single = "single",
    }
}

vocabulary! {
    /// `<hairpin>`'s `@form`: crescendo or diminuendo.
    HairpinForm {
        Cres = "cres",
        Dim = "dim",
    }
}

vocabulary! {
    /// `@form` of `<mordent>` and `<turn>`: the ornament's auxiliary note
    /// below or above the main one.
    OrnamentForm {
        Lower = "lower",
        Upper = "upper",
    }
}

vocabulary! {
    /// `<repeatMark>`'s `@func` (MEI 5).
    RepeatMarkFunc {
        Coda = "coda",
        Segno = "segno",
        DaCapo = "daCapo",
        DalSegno = "dalSegno",
        Fine = "fine",
    }
}

/// `data.KEYFIFTHS`: a key signature's accidentals, written `0`, `<n>s`
/// (sharps) or `<n>f` (flats).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeySig {
    /// Sharps positive, flats negative.
    pub fifths: i8,
}

impl KeySig {
    /// Parses a key signature; anything that isn't `<n>s`/`<n>f` reads as
    /// no accidentals.
    pub fn parse(value: &str) -> Self {
        let value = value.trim();
        let fifths = if let Some(n) = value.strip_suffix('s') {
            n.trim().parse().unwrap_or(0)
        } else if let Some(n) = value.strip_suffix('f') {
            -n.trim().parse().unwrap_or(0)
        } else {
            0
        };
        Self { fifths }
    }
}

impl fmt::Display for KeySig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.fifths.cmp(&0) {
            std::cmp::Ordering::Equal => f.write_str("0"),
            std::cmp::Ordering::Greater => write!(f, "{}s", self.fifths),
            std::cmp::Ordering::Less => write!(f, "{}f", -self.fifths),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_signatures_round_trip() {
        for (text, fifths) in [("0", 0), ("2s", 2), ("3f", -3)] {
            let key = KeySig::parse(text);
            assert_eq!(key.fifths, fifths);
            assert_eq!(key.to_string(), text);
        }
        assert_eq!(KeySig::parse("mixed").fifths, 0);
    }

    #[test]
    fn durations_and_vocabularies_keep_unknown_values() {
        assert_eq!(Duration::parse("16"), Duration::Fraction(16));
        assert_eq!(Duration::parse("breve").to_string(), "breve");
        assert_eq!(Duration::parse("x"), Duration::Other("x".into()));
        assert_eq!(Tie::parse("m"), Tie::Medial);
        assert_eq!(BarRendition::parse("heavyLight").as_str(), "heavyLight");
        assert_eq!(PitchName::parse("G"), Some(PitchName::G));
    }
}
