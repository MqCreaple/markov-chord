use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use num_derive::FromPrimitive;

use crate::{
    error,
    note::{consume_to_note, note_string, Note},
};

#[derive(FromPrimitive, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChordQuality {
    Maj,     // major
    Min,     // minor
    Dom,     // dominant
    Aug,     // augmented
    Dim,     // diminished
    HalfDim, // half diminished
}

impl ChordQuality {
    pub fn relative_pitch(&self) -> &'static [Note] {
        match self {
            ChordQuality::Maj => &[0, 4, 7, 11, 14],
            ChordQuality::Min => &[0, 3, 7, 10, 14],
            ChordQuality::Dom => &[0, 4, 7, 10, 14],
            ChordQuality::Aug => &[0, 4, 8, 12, 16],
            ChordQuality::Dim => &[0, 3, 6, 9, 12],
            ChordQuality::HalfDim => &[0, 3, 6, 10, 14],
        }
    }
}

impl Display for ChordQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            Self::Maj => "M",
            Self::Min => "m",
            Self::Dom => "dom",
            Self::Aug => "aug",
            Self::Dim => "dim",
            Self::HalfDim => "ø",
        })
    }
}

/// Defines a chord. A chord is defined from:
/// - The root note.
/// - The quality of chord (major, minor, augment, diminished, etc.).
/// - The number of notes in the chord.
///
/// The `Chord` struct does not support:
/// - Chord with additional notes, e.g. Cadd6.
/// - Chord with replaced notes, e.g. Csus2.
#[derive(PartialEq, Eq, Clone)]
pub struct Chord {
    root: Note,   // root note
    note_num: u8, // number of notes in the chord
    quality: ChordQuality,
}

impl Chord {
    /// List all notes of current chord in sequence. All notes are in modulo 12.
    fn notes(&self) -> Vec<Note> {
        self.quality
            .relative_pitch()
            .iter()
            .take(self.note_num as usize)
            .map(|rel| (self.root + rel) % 12)
            .collect()
    }
}

impl TryFrom<&str> for Chord {
    type Error = error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (root_note, key, str_next) = consume_to_note(value)?;
        let str_next_count = str_next.chars().count();
        let split_index = str_next
            .chars()
            .position(|ch| ch.is_digit(10))
            .unwrap_or(str_next_count);

        // Determine the quality of chord. Optional because quality may not be specified at this point.
        let quality_str = &str_next[0..split_index];
        let quality = match quality_str {
            "M" | "maj" => Some(ChordQuality::Maj),
            "m" | "min" => Some(ChordQuality::Min),
            "+" | "aug" => Some(ChordQuality::Aug),
            "o" | "dim" => Some(ChordQuality::Dim),
            "ø" => Some(ChordQuality::HalfDim),
            "" => None,
            _ => Err(format!("Invalid chord quality: {}", quality_str))?,
        };

        // Determine the number of notes in the chord
        let (note_num, quality_some) = if split_index == str_next_count {
            // No number indicating notes in the chord. Default number of notes depends on chord quality.
            let quality_some = quality.unwrap_or(if key {
                ChordQuality::Maj
            } else {
                ChordQuality::Min
            });
            (
                match quality_some {
                    ChordQuality::Maj
                    | ChordQuality::Min
                    | ChordQuality::Aug
                    | ChordQuality::Dim => 3,
                    ChordQuality::Dom | ChordQuality::HalfDim => 4,
                },
                quality_some,
            )
        } else {
            if let Ok(chord_num) = str_next[split_index..].parse::<u8>() {
                if chord_num % 2 == 1 {
                    (
                        (chord_num + 1) / 2,
                        quality.unwrap_or(if key {
                            ChordQuality::Dom
                        } else {
                            ChordQuality::Min
                        }),
                    )
                } else {
                    Err(format!("Invalid chord number: {}", chord_num))?
                }
            } else {
                Err("Invalid string format")?
            }
        };
        Ok(Self {
            root: root_note,
            note_num,
            quality: quality_some,
        })
    }
}

impl Display for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            note_string(self.root)
                .iter()
                .map(|note_name| if self.note_num == 3 {
                    if self.quality == ChordQuality::Maj {
                        format!("{}", note_name)
                    } else {
                        format!("{}{}", note_name, self.quality)
                    }
                } else {
                    if self.quality == ChordQuality::Dom {
                        format!("{}{}", note_name, self.note_num * 2 - 1)
                    } else {
                        format!("{}{}{}", note_name, self.quality, self.note_num * 2 - 1)
                    }
                })
                .fold(String::new(), |a, b| if a == "" { b } else { a + "/" + &b })
        )
    }
}

impl Debug for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(note: {} ({}), chord type: {:?} ",
            self.root,
            note_string(self.root).join("/"),
            self.quality
        )?;
        if self.note_num == 3 {
            write!(f, "triad)")
        } else {
            write!(f, "{})", self.note_num * 2 - 1)
        }
    }
}

impl Hash for Chord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u8(self.root);
        state.write_u8(self.quality as u8);
        state.write_u8(self.note_num);
    }
}

impl Default for Chord {
    /// Default of `Chord` struct is an A major triad.
    fn default() -> Self {
        Self {
            root: 0,
            note_num: 3,
            quality: ChordQuality::Maj,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        let c1 = Chord::try_from("C").unwrap();
        assert_eq!(c1.note_num, 3);
        assert_eq!(c1.root, 3);
        assert_eq!(c1.quality, ChordQuality::Maj);
        let c2 = Chord::try_from("Dm").unwrap();
        assert_eq!(c2.note_num, 3);
        assert_eq!(c2.root, 5);
        assert_eq!(c2.quality, ChordQuality::Min);
        let c3 = Chord::try_from("G7").unwrap();
        assert_eq!(c3.note_num, 4);
        assert_eq!(c3.root, 10);
        assert_eq!(c3.quality, ChordQuality::Dom);
        let c4 = Chord::try_from("A#dim").unwrap();
        assert_eq!(c4.note_num, 3);
        assert_eq!(c4.root, 1);
        assert_eq!(c4.quality, ChordQuality::Dim);
        let c5 = Chord::try_from("AM7").unwrap();
        assert_eq!(c5.note_num, 4);
        assert_eq!(c5.root, 0);
        assert_eq!(c5.quality, ChordQuality::Maj);
    }

    #[test]
    fn test_from_string_err() {
        let c1 = Chord::try_from("H");
        assert_eq!(c1.unwrap_err(), "Invalid note character: H");
        let c2 = Chord::try_from("Csus2");
        assert_eq!(c2.unwrap_err(), "Invalid chord quality: sus");
        let c3 = Chord::try_from("C#6");
        assert_eq!(c3.unwrap_err(), "Invalid chord number: 6");
    }

    #[test]
    fn test_notes() {
        let c_maj = Chord::try_from("C").unwrap();
        assert_eq!(c_maj.notes(), [3, 7, 10]);
        let c_maj_7 = Chord::try_from("CM7").unwrap();
        assert_eq!(c_maj_7.notes(), [3, 7, 10, 2]);
        let c_min = Chord::try_from("Cm").unwrap();
        assert_eq!(c_min.notes(), [3, 6, 10]);
        let c_min_7 = Chord::try_from("Cm7").unwrap();
        assert_eq!(c_min_7.notes(), [3, 6, 10, 1]);
        let c_dom_7 = Chord::try_from("C7").unwrap();
        assert_eq!(c_dom_7.notes(), [3, 7, 10, 1]);
        let c_aug = Chord::try_from("Caug").unwrap();
        assert_eq!(c_aug.notes(), [3, 7, 11]);
    }

    #[test]
    fn test_display() {
        let c_maj = Chord::try_from("C").unwrap();
        assert_eq!(format!("{}", c_maj), "C");
        let c_min = Chord::try_from("Cm").unwrap();
        assert_eq!(format!("{}", c_min), "Cm");
        let c_maj_7 = Chord::try_from("CM7").unwrap();
        assert_eq!(format!("{}", c_maj_7), "CM7");
        let c_min_7 = Chord::try_from("Cm7").unwrap();
        assert_eq!(format!("{}", c_min_7), "Cm7");
        let c_dom_7 = Chord::try_from("C7").unwrap();
        assert_eq!(format!("{}", c_dom_7), "C7");
    }
}
