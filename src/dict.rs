use std::io::{self, BufReader};
use std::fs::File;
use csv;
use std::u32;
use std::str::FromStr;
use aho_corasick::AcAutomaton;

#[derive(Clone, Copy, Debug, Default)]
pub struct WordEntry {
    pub word_cost: i32,
    side_id: u32,
//    pub left_id: u32,
//    pub right_id: u32,
}

impl WordEntry {
    pub fn left_id(&self) -> u32 {
        self.side_id
    }

    pub fn right_id(&self) -> u32 {
        self.side_id
    }
}

#[derive(Debug)]
pub struct CSVRow {
    surface_form: String,
    left_id: u32,
    right_id: u32,
    word_cost: i32,

    pos_level1: String,
    pos_level2: String,
    pos_level3: String,
    pos_level4: String,

    conjugation_type: String,
    conjugate_form: String,

    base_form: String,
    reading: String,
    pronunciation: String,
}

impl CSVRow {
    fn word_entry(&self) -> WordEntry {
        assert_eq!(self.left_id, self.right_id);
        WordEntry {
            word_cost: self.word_cost,
            side_id: self.left_id,
            // right_id: self.right_id,
        }
    }
}

impl<'a> From<&'a [String]> for CSVRow {
    fn from(fields: &'a [String]) -> CSVRow {
        CSVRow {
            surface_form: fields[0].clone(),
            left_id: u32::from_str(&fields[1]).expect("failed to parse left_id"),
            right_id: u32::from_str(&fields[2]).expect("failed to parse right_id"),
            word_cost: i32::from_str(&fields[3]).expect("failed to parse wordost"),

            pos_level1: fields[4].clone(),
            pos_level2: fields[5].clone(),
            pos_level3: fields[6].clone(),
            pos_level4: fields[7].clone(),

            conjugation_type: fields[8].clone(),
            conjugate_form: fields[9].clone(),

            base_form: fields[10].clone(),
            reading: fields[11].clone(),
            pronunciation: fields[12].clone(),
        }
    }
}


pub struct Dict {
    pub word_entries: Vec<WordEntry>,
    pub aho_corasick: AcAutomaton<String>,
}

impl Dict {
    pub fn get(&self, word_id: u32) -> WordEntry {
        self.word_entries[word_id as usize]
    }

    pub fn load(path: &str) -> io::Result<Dict> {
        let reader = File::open(path)?;
        let buf_reader = BufReader::new(reader);
        let mut csv_reader = csv::Reader::from_reader(buf_reader);

        let mut word_entries = vec![];
        let mut words = Vec::new();

        for result in csv_reader.records() {
            let record = result.unwrap();
            let row = CSVRow::from(&record[..]);
            word_entries.push(row.word_entry());
            words.push(row.surface_form);
        }

        Ok(Dict {
            word_entries: word_entries,
            aho_corasick: AcAutomaton::new(words),
        })
    }
}