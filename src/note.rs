use crate::error::Result;

pub type Note = u8; // 0 as note 'A' or roman numeral 'I'

/// Read a note from a string slice. Returns the note number, key (major = true, minor = false), and
/// the position just after the note.
pub(crate) fn consume_to_note(str: &str) -> Result<(Note, bool, &str)> {
    let mut index = 0;
    if let Some(note_char) = str.chars().nth(index) {
        let mut note = match note_char {
            'A' => Ok(0),
            'B' => Ok(2),
            'C' => Ok(3),
            'D' => Ok(5),
            'E' => Ok(7),
            'F' => Ok(8),
            'G' => Ok(10),
            _ => Err(format!("Invalid note character: {}", note_char)),
        }?;
        let key = note_char.is_uppercase();
        index += 1;
        match str.chars().nth(index) {
            Some('#') | Some('♯') => {
                index += 1;
                note += 1;
            }
            Some('b') | Some('♭') => {
                index += 1;
                note -= 1;
            }
            Some('♮') => {
                index += 1;
            }
            _ => {}
        }
        Ok((note, key, &str[index..]))
    } else {
        Err("Invalid note format".to_string())
    }
}

pub fn note_string(note: Note) -> Vec<&'static str> {
    match note % 12 {
        0 => vec!["A"],
        1 => vec!["A#", "Bb"],
        2 => vec!["B"],
        3 => vec!["C"],
        4 => vec!["C#", "Db"],
        5 => vec!["D"],
        6 => vec!["D#", "Eb"],
        7 => vec!["E"],
        8 => vec!["F"],
        9 => vec!["F#", "Gb"],
        10 => vec!["G"],
        11 => vec!["G#", "Ab"],
        _ => vec![],
    }
}
