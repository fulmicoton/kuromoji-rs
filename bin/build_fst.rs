use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::str::FromStr;
use std::u32;

extern crate byteorder;
extern crate fst;
use fst::MapBuilder;

mod connection;
mod word_entry;
use crate::word_entry::WordEntry;

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

impl<'a> From<&'a [&'a str]> for CSVRow {
    fn from(fields: &'a [&'a str]) -> CSVRow {
        CSVRow {
            surface_form: fields[0].to_string(),
            left_id: u32::from_str(&fields[1]).expect("failed to parse left_id"),
            right_id: u32::from_str(&fields[2]).expect("failed to parse right_id"),
            word_cost: i32::from_str(&fields[3]).expect("failed to parse wordost"),

            pos_level1: fields[4].to_string(),
            pos_level2: fields[5].to_string(),
            pos_level3: fields[6].to_string(),
            pos_level4: fields[7].to_string(),

            conjugation_type: fields[8].to_string(),
            conjugate_form: fields[9].to_string(),

            base_form: fields[10].to_string(),
            reading: fields[11].to_string(),
            pronunciation: fields[12].to_string(),
        }
    }
}

fn convert() -> io::Result<()> {
    let file = File::open("./dict/dict.csv")?;
    let buffer = BufReader::new(file);
    let mut rows = Vec::new();
    for line_res in buffer.lines() {
        let line = line_res?;
        let fields: Vec<&str> = line.split(",").collect();
        let row = CSVRow::from(&fields[..]);
        rows.push(row);
    }
    rows.sort_by_key(|row| row.surface_form.clone());

    let wtr = io::BufWriter::new(File::create("dict/dict.fst")?);
    let mut build = MapBuilder::new(wtr).unwrap();

    let mut multiple_id: u8 = 0u8;
    let mut previous = String::from("");

    for row in &rows {
        let word_entry = WordEntry {
            word_cost: row.word_cost,
            cost_id: row.left_id,
        };
        assert_eq!(
            WordEntry::decode_from_u64(word_entry.encode_as_u64()),
            word_entry
        );
        let key = &row.surface_form;
        if key != &previous {
            previous = key.clone();
            multiple_id = 0;
        }
        let mut extended_key = Vec::from(key.as_bytes());
        extended_key.push(0);
        extended_key.push(multiple_id as u8);
        assert!(multiple_id < 100);
        build.insert(extended_key, word_entry.encode_as_u64());
        multiple_id += 1u8;
    }

    build.finish().unwrap();
    Ok(())
}

fn main() {
    convert().expect("done");
}
