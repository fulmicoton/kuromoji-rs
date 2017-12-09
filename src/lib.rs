extern crate fst;
extern crate byteorder;


mod viterbi;
mod connection;
mod word_entry;

use self::word_entry::WordEntry;
use self::viterbi::Lattice;
use self::connection::ConnectionCostMatrix;

const DICTIONARY_DATA: &'static [u8] = include_bytes!("../dict/dict.fst");


pub struct Tokenizer {
    dict: fst::raw::Fst,
    cost_matrix: ConnectionCostMatrix,
    lattice: Lattice,
}


impl Tokenizer {

    pub fn new() -> Tokenizer {
        let dict = fst::raw::Fst::from_static_slice(DICTIONARY_DATA).unwrap();
        let cost_matrix = ConnectionCostMatrix::load_default();
        Tokenizer {
            dict: dict,
            cost_matrix: cost_matrix,
            lattice: Lattice::default(),
        }
    }

    pub fn tokenize(&mut self, text: &str) -> Vec<usize> {
        self.lattice.set_text(&self.dict, text);
        self.lattice.calculate_path_costs(&self.cost_matrix);
        let tokens_offset = self.lattice.tokens_offset();
        tokens_offset
    }
}


#[cfg(test)]
mod tests {

    use super::Tokenizer;
    use fst;
    use fst::Streamer;
    use super::*;
    use std::str;

    #[test]
    fn test_dict() {
        let mut tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("俺はまだ本気出してないだけ。");
        assert_eq!(tokens, vec![0, 3, 6, 12, 18, 24, 27, 33, 39]);
    }
}
