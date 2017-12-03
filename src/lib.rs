extern crate aho_corasick;
extern crate csv;

mod viterbi;
mod dict;
mod connection;

use self::viterbi::Lattice;
use self::dict::Dict;
use self::connection::ConnectionCostMatrix;

pub struct Tokenizer {
    dict: Dict,
    cost_matrix: ConnectionCostMatrix,
    lattice: Lattice,
}


impl Tokenizer {

    pub fn new() -> Tokenizer {
        let dict = Dict::load("/Users/pmasurel/github/kuromoji/ipadic-utf8/total.csv").unwrap();
        let cost_matrix = ConnectionCostMatrix::load("/Users/pmasurel/github/kuromoji/ipadic-utf8/matrix.def").unwrap();
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

    use dict::Dict;
    use super::Tokenizer;

    #[test]
    fn test_dict() {
        let mut tokenizer = Tokenizer::new();
        for i in 0..10000000 {
            let tokens = tokenizer.tokenize("すもももももももものうち");
            assert!(tokens.len() > 0);
        }
    }
}