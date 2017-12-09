use std::io::{self, BufRead, BufReader, Write};
use std::fs::File;
use std::u32;
use std::str::FromStr;

extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};


fn convert() -> io::Result<()> {
    let file = File::open("./dict/matrix.def")?;
    let buffer = BufReader::new(file);
    let mut lines = buffer.lines().map(|line_res| line_res
        .unwrap()
        .split(' ')
        .map(|s| i32::from_str(s).unwrap())
        .collect::<Vec<i32>>());
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