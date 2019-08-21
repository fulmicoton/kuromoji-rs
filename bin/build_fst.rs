//mod connection;
//mod word_entry;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::str::FromStr;
use std::u32;
use tantivy_fst::MapBuilder;
use kuromoji::WordEntry;
use std::collections::BTreeMap;

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

    let wtr_fst = io::BufWriter::new(File::create("dict/dict.fst")?);
    let mut wtr_vals = io::BufWriter::new(File::create("dict/dict.vals")?);

    let mut word_entry_map: BTreeMap<String, Vec<WordEntry>> = BTreeMap::new();

    for row in &rows {
        word_entry_map
            .entry(row.surface_form.clone())
            .or_insert_with(Vec::new)
            .push(WordEntry {
                word_cost: row.word_cost,
                cost_id: row.left_id,
            });
    }

    let mut id = 0u64;
    let mut fst_build = MapBuilder::new(wtr_fst).unwrap();
    for (key, word_entries) in &word_entry_map {
        let len = word_entries.len() as u64;
        assert!(len < (1 << 5));
        let val = (id << 5) | len;
        dbg!(val);
        fst_build.insert(&key, val).unwrap();
        id += len;
    }

    for word_entries in word_entry_map.values() {
        for word_entry in word_entries {
            word_entry.serialize(&mut wtr_vals)?;
        }
    }
    wtr_vals.flush()?;

    fst_build.finish().unwrap();
    Ok(())
}

fn main() -> io::Result<()> {
    convert()
}
