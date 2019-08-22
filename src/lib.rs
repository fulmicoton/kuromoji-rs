mod connection;
mod viterbi;
mod word_entry;
mod prefix_dict;
mod character_definition;
mod unknown_dictionary;

use std::io;
use encoding::DecoderTrap;
use crate::connection::ConnectionCostMatrix;
use crate::viterbi::{Lattice, Edge};
pub use crate::word_entry::WordEntry;
use crate::prefix_dict::PrefixDict;
pub use crate::character_definition::{CharacterDefinitions, CharacterDefinitionsBuilder};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use encoding::Encoding;
use std::num::ParseIntError;
use crate::unknown_dictionary::UnknownDictionary;

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


#[derive(Clone, Debug)]
pub struct Penalty {
    kanji_penalty_length_threshold: usize,
    kanji_penalty_length_penalty: i32,
    other_penalty_length_threshold: usize,
    other_penalty_length_penalty: i32,
}

impl Default for Penalty {
    fn default() -> Self {
        Penalty {
            kanji_penalty_length_threshold: 2,
            kanji_penalty_length_penalty: 3000,
            other_penalty_length_threshold: 7,
            other_penalty_length_penalty: 1700,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    Normal,
    Search(Penalty),
}

impl Penalty {
    pub fn penalty(&self, edge: &Edge) -> i32 {
        let num_chars = edge.num_chars();
        if num_chars <= self.kanji_penalty_length_threshold {
            return 0;
        }
        if edge.kanji_only {
            ((num_chars - self.kanji_penalty_length_threshold) as i32) * self.kanji_penalty_length_penalty
        } else if num_chars > self.other_penalty_length_threshold {
            ((num_chars - self.other_penalty_length_threshold) as i32) * self.other_penalty_length_penalty
        } else {
            0
        }
    }
}


impl Mode {

    pub fn is_search(&self) -> bool {
        match self {
            Mode::Normal => false,
            Mode::Search(_penalty) => true,
        }
    }
    pub fn penalty_cost(&self, edge: &Edge) -> i32 {
        match self {
            Mode::Normal => 0i32,
            Mode::Search(penalty) => penalty.penalty(edge),
        }
    }
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


pub struct Tokenizer {
    dict: PrefixDict<&'static [u8]>,
    cost_matrix: ConnectionCostMatrix,
    lattice: Lattice,
    char_definitions: CharacterDefinitions,
    unknown_dictionary: UnknownDictionary,
    mode: Mode,
    offsets: Vec<usize>,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        let dict = PrefixDict::default();
        let cost_matrix = ConnectionCostMatrix::load_default();
        let char_definitions = CharacterDefinitions::load();
        let unknown_dictionary =
            UnknownDictionary::load(&char_definitions).unwrap();
        Tokenizer {
            dict,
            cost_matrix,
            lattice: Lattice::default(),
            char_definitions,
            unknown_dictionary,
            mode: Mode::Search(Penalty::default()),
            offsets: Vec::new()
        }
    }

    pub fn tokenize_offsets(&mut self, text: &str) -> &[usize] {
        self.lattice.set_text(&self.dict, &self.char_definitions, &self.unknown_dictionary, text, &self.mode);
        self.lattice.calculate_path_costs(&self.cost_matrix, &self.mode);
        self.lattice.tokens_offset(&mut self.offsets);
        &self.offsets[..]
    }

    pub fn tokenize<'a>(&'a mut self, text: &'a str) -> impl Iterator<Item=&'a str> + 'a {
        self.tokenize_offsets(text);
        self.offsets.push(text.len());
        self.offsets
            .windows(2)
            .map(move|arr| &text[arr[0]..arr[1]])
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
    fn test_tokenize() {
        let mut tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize_offsets("俺はまだ本気出してないだけ。");
        assert_eq!(tokens, &[0, 3, 6, 12, 18, 24, 27, 33, 39]);
    }

    #[test]
    fn test_tokenize2() {
        let mut tokenizer = Tokenizer::new();
        let tokens: Vec<&str> = tokenizer.tokenize("私の名前はマズレル野恵美です。").collect();
        assert_eq!(tokens, vec!["私", "の", "名前", "は", "マズレル", "野", "恵美", "です", "。"]);
    }

    #[test]
    fn test_tokenize_junk() {
        let mut tokenizer = Tokenizer::new();
        let tokens: Vec<&str> = tokenizer.tokenize("関西国werwerママママ空港").collect();
        assert_eq!(tokens, vec!["関西", "国", "werwer", "ママ", "ママ", "空港"]);
    }

    #[test]
    fn test_tokenize_search_mode() {
        let mut tokenizer = Tokenizer::new();
        let tokens: Vec<&str> = tokenizer.tokenize("関西国際空港").collect();
        assert_eq!(tokens, vec!["関西", "国際", "空港"]);
    }

    #[test]
    fn test_tokenize_sumomomomo() {
        let mut tokenizer = Tokenizer::new();
        let tokens: Vec<&str> = tokenizer.tokenize("すもももももももものうち").collect();
        assert_eq!(tokens, vec!["すもも", "も", "もも", "も", "もも", "の", "うち"]);
    }


}
