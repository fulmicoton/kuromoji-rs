mod connection;
mod viterbi;
mod word_entry;
mod prefix_dict;
pub mod character_definition;
pub mod unknown_dictionary;

use std::io;
use encoding::DecoderTrap;
use crate::connection::ConnectionCostMatrix;
use crate::viterbi::{Lattice, Edge};
pub use crate::word_entry::{WordEntry, WordDetail};
use crate::prefix_dict::PrefixDict;
pub use crate::character_definition::CharacterDefinitions;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use encoding::Encoding;
use std::num::ParseIntError;
use crate::unknown_dictionary::UnknownDictionary;
use std::fmt::Debug;
use crate::word_entry::WordDictionary;

#[derive(Debug)]
pub enum ParsingError {
    Encoding,
    IoError(io::Error),
    ContentError(String)
}


impl ParsingError {
    fn from_error<D: Debug>(error: D) -> ParsingError {
        ParsingError::ContentError(format!("{:?}", error))
    }
}

impl From<io::Error> for ParsingError {
    fn from(io_err: io::Error) -> Self {
        ParsingError::IoError(io_err)
    }
}

impl From<ParseIntError> for ParsingError {
    fn from(parse_err: ParseIntError) -> Self {
        ParsingError::from_error(parse_err)
    }
}


fn normalize_string(s: &str, output: &mut String) {
    
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

pub fn read_mecab_file(filename: &'static str) -> Result<String, ParsingError> {
    let path = Path::new( "mecab-ipadic").join(Path::new(filename));
    let mut input_read = File::open(path)?;
    let mut buffer = Vec::new();
    input_read.read_to_end(&mut buffer)?;
    encoding::all::EUC_JP.decode(&buffer, DecoderTrap::Strict)
        .map_err(|_| ParsingError::Encoding)
}


pub struct Token<'a> {
    pub text: &'a str,
    pub detail: WordDetail
}

pub struct Tokenizer {
    dict: PrefixDict<&'static [u8]>,
    cost_matrix: ConnectionCostMatrix,
    lattice: Lattice,
    char_definitions: CharacterDefinitions,
    unknown_dictionary: UnknownDictionary,
    mode: Mode,
    offsets: Vec<(usize, u32)>,
}

impl Tokenizer {
    pub fn new(mode: Mode) -> Tokenizer {
        let dict = PrefixDict::default();
        let cost_matrix = ConnectionCostMatrix::load_default();
        let char_definitions = CharacterDefinitions::load();
        let unknown_dictionary = UnknownDictionary::load();
        Tokenizer {
            dict,
            cost_matrix,
            lattice: Lattice::default(),
            char_definitions,
            unknown_dictionary,
            mode,
            offsets: Vec::new()
        }
    }

    pub fn for_search() -> Tokenizer {
        Self::new(Mode::Search(Penalty::default()))
    }


    pub fn normal() -> Tokenizer {
        Self::new(Mode::Normal)
    }


    /// Returns an array of offsets that mark the beginning of each tokens,
    /// in bytes.
    ///
    /// For instance
    /// e.g. "僕は'
    ///
    /// returns the array `[0, 3]`
    ///
    /// The array, always starts with 0, except if you tokenize the empty string,
    /// in which case an empty array is returned.
    ///
    /// Whitespaces also count as tokens.
    pub(crate) fn tokenize_offsets(&mut self, text: &str) -> &[(usize, u32)] {
        if text.is_empty() {
            return &[];
        }
        self.lattice.set_text(&self.dict, &self.char_definitions, &self.unknown_dictionary, text, &self.mode);
        self.lattice.calculate_path_costs(&self.cost_matrix, &self.mode);
        self.lattice.tokens_offset(&mut self.offsets);
        &self.offsets[..]
    }

    fn tokenize_without_split<'a>(&mut self, text: &'a str, tokens: &mut Vec<Token<'a>>) {
        let offsets = self.tokenize_offsets(text);
        for i in 0..offsets.len() {
            let (token_start, word_detail) = offsets[i];
            let token_stop =
                if i == offsets.len() - 1 {
                    text.len()
                } else {
                    let (next_start, _) = offsets[i + 1];
                    next_start
                };
            tokens.push(Token {
                text: &text[token_start..token_stop],
                detail: WordDictionary::load_word_id(word_detail)
            })
        }
    }

    fn tokenize_without_split_str<'a>(&mut self, text: &'a str, tokens: &mut Vec<&'a str>) {
        let offsets = self.tokenize_offsets(text);
        for i in 0..offsets.len() {
            let (token_start, _word_detail) = offsets[i];
            let token_stop =
                if i == offsets.len() - 1 {
                    text.len()
                } else {
                    let (next_start, _) = offsets[i + 1];
                    next_start
                };
            tokens.push(&text[token_start..token_stop]);
        }
    }

    pub fn tokenize<'a>(&'a mut self, mut text: &'a str) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(split_idx) = text.find(|c| c=='。' || c == '、') {
            self.tokenize_without_split(&text[..split_idx + 3], &mut tokens);
            text = &text[split_idx+3..];
        }
        if !text.is_empty() {
            self.tokenize_without_split(&text, &mut tokens);
        }
        tokens
    }

    pub fn tokenize_str<'a>(&'a mut self, mut text: &'a str) -> Vec<&'a str> {
        let mut tokens = Vec::new();
        while let Some(split_idx) = text.find(|c| c=='。' || c == '、') {
            self.tokenize_without_split_str(&text[..split_idx + 3], &mut tokens);
            text = &text[split_idx+3..];
        }
        if !text.is_empty() {
            self.tokenize_without_split_str(&text, &mut tokens);
        }
        tokens
    }
}


#[cfg(test)]
mod tests {

    use super::Tokenizer;

    #[test]
    fn test_empty() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens = tokenizer.tokenize_offsets("");
        assert_eq!(tokens, &[]);
    }

    /*
    #[test]
    fn test_space() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens = tokenizer.tokenize_offsets(" ");
        assert_eq!(tokens, &[0]);
    }


    #[test]
    fn test_boku_ha() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens = tokenizer.tokenize_offsets("僕は");
        assert_eq!(tokens, &[0, 3]);
    }

    #[test]
    fn test_tokenize() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens = tokenizer.tokenize_offsets("俺はまだ本気出してないだけ。");
        assert_eq!(tokens, &[0, 3, 6, 12, 18, 24, 27, 33, 39]);
    }

    #[test]
    fn test_tokenize2() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens: Vec<&str> = tokenizer.tokenize_str("私の名前はマズレル野恵美です。");
        assert_eq!(tokens, vec!["私", "の", "名前", "は", "マズレル", "野", "恵美", "です", "。"]);
    }

    #[test]
    fn test_tokenize_junk() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens: Vec<&str> = tokenizer.tokenize_str("関西国werwerママママ空港");
        assert_eq!(tokens, vec!["関西", "国", "werwer", "ママ", "ママ", "空港"]);
    }

    #[test]
    fn test_tokenize_search_mode() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens: Vec<&str> = tokenizer.tokenize_str("関西国際空港");
        assert_eq!(tokens, vec!["関西", "国際", "空港"]);
    }

    #[test]
    fn test_tokenize_sumomomomo() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens: Vec<&str> = tokenizer.tokenize_str("すもももももももものうち");
        assert_eq!(tokens, vec!["すもも", "も", "もも", "も", "もも", "の", "うち"]);
    }

    #[test]
    fn test_mukigen_search() {
        let mut tokenizer = Tokenizer::for_search();
        let tokens: Vec<&str> = tokenizer.tokenize_str("無期限に—でもどの種を?");
        assert_eq!(tokens, vec!["無", "期限", "に", "—", "でも", "どの", "種", "を", "?"]);
    }
    */

    #[test]
    fn test_gyoi() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("御意。 御意〜。");
        assert_eq!(tokens, vec!["御意", "。", " ", "御意", "〜。"]);
    }

    #[test]
    fn test_demoyorokobi() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("〜でも喜び");
        assert_eq!(tokens, vec!["〜", "でも", "喜び"]);
    }

    #[test]
    fn test_mukigen_normal2() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("—でも");
        assert_eq!(tokens, vec!["—", "でも"]);
    }

    #[test]
    fn test_atodedenwa() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("−後で");
        assert_eq!(tokens, vec!["−", "後で"]);
    }

    #[test]
    fn test_ikkagetsu() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("ーヶ月");
        assert_eq!(tokens, vec!["ーヶ", "月"]);
    }

    #[test]
    fn test_mukigen_normal() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("無期限に—でもどの種を?");
        assert_eq!(tokens, vec!["おはよう", "一", "夏休み", "さ", " ", "あと", "お", "ー", "ヶ月", " ", "ぐらい", "ほしい", "よ", "な"]);
    }

    #[test]
    fn test_demo() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("――!!?");
        assert_eq!(tokens, vec!["――!!?"]);
    }

    #[test]
    fn test_kaikeishi() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("ジム・コガン");
        assert_eq!(tokens, vec!["ジム・コガン"]);
    }

    #[test]
    fn test_bruce() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("ブルース・モラン");
        assert_eq!(tokens, vec!["ブルース・モラン"]);
    }

    #[test]
    fn test_tokenize_real() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str(
            "本項で解説する地方病とは、山梨県における日本住血吸虫症の呼称であり、\
            長い間その原因が明らかにならず住民を苦しめた感染症である。");
        assert_eq!(tokens, vec!["本", "項", "で", "解説", "する", "地方",
                                "病", "と", "は", "、", "山梨", "県", "における",
                                "日本", "住", "血", "吸", "虫", "症", "の",
                                "呼称", "で", "あり", "、", "長い", "間", "その", "原因", "が", "明らか", "に", "なら", "ず", "住民", "を", "苦しめ", "た", "感染", "症", "で", "ある", "。"]);
    }

    #[test]
    fn test_hitobito() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize_str("満々!");
        assert_eq!(tokens, &["満々", "!"]);
    }
    /*
    #[test]
    fn test_tokenize_short() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize(
            "日本住").collect();
        assert_eq!(tokens, vec!["日本", "住"]);
    }

    #[test]
    fn test_tokenize_short2() {
        let mut tokenizer = Tokenizer::normal();
        let tokens: Vec<&str> = tokenizer.tokenize(
            "ここでは").collect();
        assert_eq!(tokens, vec!["ここ", "で", "は"]);
    }
    */

}
