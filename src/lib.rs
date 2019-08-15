mod connection;
mod viterbi;
mod word_entry;
mod prefix_dict;
mod character_definition;
mod unknown_dictionary;

use std::io;
use encoding::DecoderTrap;

const IPADIC_PATH: &'static str = "ipadic/mecab-ipadic-2.7.0-20070801";

pub(crate) fn ipadic_path()-> &'static Path {
    Path::new(IPADIC_PATH)
}

#[derive(Debug)]
pub enum ParsingError {
    Encoding,
    IoError(io::Error),
    ContentError
}

impl From<ParseIntError> for ParsingError {
    fn from(_parse_err: ParseIntError) -> Self {
        ParsingError::ContentError
    }
}

impl From<io::Error> for ParsingError {
    fn from(io_err: io::Error) -> Self {
        ParsingError::IoError(io_err)
    }
}

pub(crate) fn read_all(path: &Path) -> Result<String, ParsingError> {
    let mut input_read = File::open(path)?;
    let mut buffer = Vec::new();
    input_read.read_to_end(&mut buffer)?;
    encoding::all::EUC_JP.decode(&buffer, DecoderTrap::Strict)
        .map_err(|_| ParsingError::Encoding)
}

use crate::connection::ConnectionCostMatrix;
use crate::viterbi::Lattice;
pub(crate) use crate::word_entry::WordEntry;
use crate::prefix_dict::PrefixDict;
pub use crate::character_definition::{CharacterDefinitions, CharacterDefinitionsBuilder};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use encoding::Encoding;
use std::num::ParseIntError;
use crate::unknown_dictionary::UnknownDictionary;

const DICTIONARY_DATA: &'static [u8] = include_bytes!("../dict/dict.fst");

pub struct Tokenizer {
    dict: PrefixDict,
    cost_matrix: ConnectionCostMatrix,
    lattice: Lattice,
    char_definitions: CharacterDefinitions,
    unknown_dictionary: UnknownDictionary,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        let dict = PrefixDict::from_static_slice(DICTIONARY_DATA).unwrap();
        let cost_matrix = ConnectionCostMatrix::load_default();
        let char_definitions = CharacterDefinitions::load();
        let unknown_dictionary =
            UnknownDictionary::load(&char_definitions).unwrap();
        Tokenizer {
            dict,
            cost_matrix,
            lattice: Lattice::default(),
            char_definitions,
            unknown_dictionary
        }
    }

    pub fn tokenize(&mut self, text: &str) -> Vec<usize> {
        self.lattice.set_text(&self.dict, &self.char_definitions, &self.unknown_dictionary, text, true);
        self.lattice.calculate_path_costs(&self.cost_matrix);
        let tokens_offset = self.lattice.tokens_offset();
        tokens_offset
    }
}

impl Default for Tokenizer {
    fn default() -> Tokenizer {
        Tokenizer::new()
    }
}

#[cfg(test)]
mod tests {

    use super::Tokenizer;

    #[test]
    fn test_dict() {
        let mut tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("俺はまだ本気出してないだけ。");
        assert_eq!(tokens, vec![0, 3, 6, 12, 18, 24, 27, 33, 39]);
    }
}
