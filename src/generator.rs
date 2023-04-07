use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};
use rand::{Rng, distributions::WeightedIndex};

use crate::{chord::Chord, error::Result};

pub struct ChordGenerator {
    map_forward: Vec<Chord>,
    map_backward: HashMap<Chord, usize>,
    transit: DMatrix<f32>,
    transit_pow_cache: HashMap<u32, DMatrix<f32>>,
}

impl ChordGenerator {
    pub fn new(chord_seq: &[Chord]) -> Self {
        let mut map_forward = Vec::new();
        let mut map_backward: HashMap<Chord, usize> = HashMap::new();
        for chord in chord_seq {
            if !map_backward.contains_key(&chord) {
                map_backward.insert(chord.clone(), map_forward.len());
                map_forward.push(chord.clone());
            }
        }
        let mut cooccur: DMatrix<f32> = DMatrix::zeros(map_forward.len(), map_forward.len());
        for i in 1..chord_seq.len() {
            cooccur[(map_backward[&chord_seq[i]], map_backward[&chord_seq[i-1]])] += 1.0;
        }
        cooccur[(map_backward[&chord_seq[0]], map_backward[&chord_seq[chord_seq.len() - 1]])] += 1.0;
        for i in 0..map_forward.len() {
            cooccur.set_column(i, &(cooccur.column(i) / cooccur.column(i).sum()));
        }
        Self { map_forward, map_backward, transit: cooccur, transit_pow_cache: HashMap::new() }
    }

    /// Generate a sequence of chords with length `number` with plain Markov chain model, or return
    /// an error.
    pub fn generate(&self, init_chord: Chord, number: usize, rng: &mut impl Rng) -> Result<Vec<Chord>> {
        let mut ans = Vec::with_capacity(number);
        let mut cur_chord_index = self.map_backward[&init_chord];
        for _ in 0..number {
            let column = self.transit.column(cur_chord_index);
            let probability = column.as_slice();
            if let Ok(distr) = WeightedIndex::new(probability) {
                let gen = rng.sample(distr);
                let gened_chord = self.map_forward[gen].clone();
                ans.push(gened_chord.clone());
                cur_chord_index = gen;
            } else {
                return Err("No chords are stored in the gererator!".to_string());
            }
        }
        Ok(ans)
    }

    /// Give the chord at index `0` and `right_index`, returns the probability vector of chord at `gen_index`.
    pub fn probability_on(&mut self, left_chord: Chord, right_chord: Chord, right_index: usize, gen_index: usize) -> Result<DVector<f32>> {
        if right_index <= gen_index {
            return Err(format!("Right index {} is not greater than {}, the index of chord being generated", right_index, gen_index));
        }
        match (self.map_backward.get(&left_chord), self.map_backward.get(&right_chord)) {
            (Some(&l_ch), Some(&r_ch)) => {
                let p_r_g = self.transit_pow((right_index - gen_index) as u32);
                let p_g = self.transit_pow(gen_index as u32);
                let p_r = self.transit_pow(right_index as u32);
                Ok(p_g.column(l_ch).component_mul(&p_r_g.row(r_ch).transpose()) / p_r[(r_ch, l_ch)])
            },
            (None, _) => {
                Err(format!("Chord {} not appeared in training set.", left_chord))
            },
            (_, None) => {
                Err(format!("Chord {} not appeared in training set.", right_chord))
            },
        }
    }

    /// Give the chord at index -1 and `ans_vec.len()`, fill the mutable chord array's index 0 (inclusive)
    /// to `ans_vec.len() - 1` (inclusive) with randomly generated chords or returns an error.
    fn generate_fill(&mut self, ans_vec: &mut [Chord], left_chord: Chord, right_chord: Chord, rng: &mut impl Rng) -> Result<()> {
        if ans_vec.len() == 0 {
            return Ok(())
        }
        let mid = ans_vec.len() / 2;

        // random select
        let probability: Vec<f32> = self.probability_on(left_chord.clone(), right_chord.clone(), ans_vec.len() + 1, mid + 1)?
                .data.into();
        if let Ok(distr) = WeightedIndex::new(probability) {
            let gen = rng.sample(distr);
            let gened_chord = self.map_forward[gen].clone();
            ans_vec[mid] = gened_chord.clone();
            self.generate_fill(&mut ans_vec[0..mid], left_chord, gened_chord.clone(), rng)?;
            self.generate_fill(&mut ans_vec[(mid + 1)..], gened_chord.clone(), right_chord.clone(), rng)?;
            Ok(())
        } else {
            Err("No chords are stored in the gererator!".to_string())
        }
    }

    /// Give the chord at index 0 and `right_index`, returns a randomly generated sequence between
    /// index 1 (inclusive) and `right_index - 1` (inclusive) or returns an error.
    pub fn generate_range(&mut self, left_chord: Chord, right_chord: Chord, right_index: usize, rng: &mut impl Rng) -> Result<Vec<Chord>> {
        let mut ans = vec![Chord::default(); right_index - 2];
        self.generate_fill(&mut ans, left_chord, right_chord, rng)?;
        Ok(ans)
    }

    /// Get the nth power of transition matrix.
    /// 
    /// If the nth power is cached, directly return the cached matrix. Otherwise, calculate it using
    /// binary exponent algorithm.
    fn transit_pow(&mut self, n: u32) -> DMatrix<f32> {
        if n == 1 {
            self.transit.clone()
        } else if let Some(pow_n) = self.transit_pow_cache.get(&n) {
            pow_n.clone()
        } else {
            let pow_n_2 = self.transit_pow(n / 2);
            let ans = if n & 1 > 0 {
                pow_n_2.clone() * pow_n_2 * &self.transit
            } else {
                pow_n_2.clone() * pow_n_2
            };
            self.transit_pow_cache.insert(n, ans.clone());
            ans
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let chord_seq1 = [
            Chord::try_from("C").unwrap(),
            Chord::try_from("G").unwrap(),
            Chord::try_from("Am").unwrap(),
            Chord::try_from("F").unwrap(),
        ];
        let cg1 = ChordGenerator::new(&chord_seq1);
        assert_eq!(cg1.map_forward, chord_seq1);
        assert_eq!(cg1.transit[(2, 1)], 1.0);
        assert_eq!(cg1.transit[(0, 3)], 1.0);

        let chord_seq2 = [
            Chord::try_from("C").unwrap(),
            Chord::try_from("G").unwrap(),
            Chord::try_from("Am").unwrap(),
            Chord::try_from("Em").unwrap(),
            Chord::try_from("F").unwrap(),
            Chord::try_from("C").unwrap(),
            Chord::try_from("F").unwrap(),
            Chord::try_from("G").unwrap(),
        ];
        let cg2 = ChordGenerator::new(&chord_seq2);
        assert_eq!(cg2.transit[(0, 4)], 0.5);
        assert_eq!(cg2.transit[(1, 4)], 0.5);
    }
}