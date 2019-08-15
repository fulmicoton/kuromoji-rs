use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::str::FromStr;
use std::{i32, u32};

use byteorder::{LittleEndian, WriteBytesExt};
use crate::ParsingError;

fn convert() -> Result<(), ParsingError> {
    let file = File::open("./dict/matrix.def")?;
    let buffer = BufReader::new(file);
    let mut lines = Vec::new();
    for line_res in buffer.lines() {
        let line = line_res?;
        let fields: Vec<i32> = line.split_whitespace()
            .map(i32::from_str)
            .collect::<Result<_>()?;
        lines.push(fields);
    }
    let header = lines.next().unwrap();
    let forward_size = header[0] as u32;
    let backward_size = header[1] as u32;
    let len = 2 + (forward_size * backward_size) as usize;
    let mut costs = vec![i16::max_value(); len];
    costs[0] = forward_size as i16;
    costs[1] = backward_size as i16;
    for fields in lines {
        let forward_id = fields[0] as u32;
        let backward_id = fields[1] as u32;
        let cost = fields[2] as u16;
        costs[2 + (backward_id + forward_id * backward_size) as usize] = cost as i16;
    }

    let mut wtr = io::BufWriter::new(File::create("dict/matrix.mtx")?);
    for cost in costs {
        wtr.write_i16::<LittleEndian>(cost)?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() {
    convert().expect("done");
}
