use std::io::{self, Read, BufRead, BufReader};
use std::fs::File;
use std::u32;
use std::str::FromStr;
use fst;

const DICTIONARY_DATA: &'static [u8] = include_bytes!("../dict/dict.fst");

pub struct Dict {
    pub fst: fst::Map,
}

impl Dict {
//
//    pub fn get(&self, word_id: u32) -> WordEntry {
//        self.word_entries[word_id as usize]
//    }

    pub fn load_default() -> Dict {
        Dict {
            fst: fst::raw::from_static_slice(DICTIONARY_DATA),
        }
    }

//    pub fn load_csv<R: BufRead>(read: R) -> io::Result<Dict> {
//        let mut word_entries = vec![];
//        let mut words = Vec::new();
//        for line_res in read.lines() {
//            let line = line_res?;
//            let fields: Vec<&str> = line.split(",").collect();
//            let row = CSVRow::from(&fields[..]);
//            word_entries.push(row.word_entry());
//            words.push(row.surface_form);
//        }
//        Ok(Dict {
//            word_entries: word_entries,
//            aho_corasick: AcAutomaton::new(words),
//        })
//    }
//
//    pub fn load_file(path: &str) -> io::Result<Dict> {
//        let reader = File::open(path)?;
//        let buf_reader = BufReader::new(reader);
//        Dict::load_csv(buf_reader)
//    }
}

