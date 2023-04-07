use std::{fs::File, io::Read};

use chord::Chord;
use generator::ChordGenerator;

mod chord;
mod error;
mod generator;
mod note;

fn read_generator(file: &mut File) -> ChordGenerator {
    let mut file_string = String::new();
    file.read_to_string(&mut file_string).unwrap();
    let chord_seq = file_string
        .split([',', '|', ' '])
        .filter(|s| !s.is_empty())
        .map(|s| Chord::try_from(s).unwrap())
        .collect::<Vec<_>>();
    ChordGenerator::new(&chord_seq)
}

fn main() {
    let mut generator = read_generator(&mut File::open("chord.txt").unwrap());
    let left_chord = Chord::try_from("F").unwrap();
    let right_chord = Chord::try_from("C").unwrap();
    let generated = generator.generate_range(
        left_chord.clone(),
        right_chord.clone(),
        8,
        &mut rand::thread_rng(),
    );
    // let generated = generator.generate(
    //     left_chord.clone(), 16, &mut rand::thread_rng()
    // );
    print!("{} ", left_chord);
    for chord in generated.unwrap() {
        print!("{} ", chord);
    }
    println!("{} ", right_chord);
}
