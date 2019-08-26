use std::env;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufRead, BufWriter, Write};
use kuromoji::Tokenizer;

fn main() -> io::Result<()> {
    let mut args_it = env::args();
    let _ = args_it.next().unwrap();
    let input = args_it.next().unwrap();
    let output = args_it.next().unwrap();
    let f = File::open(input)?;
    let buff = BufReader::new(f);

    let mut wtr = BufWriter::new(File::create(output)?);
    let mut tokenizer = Tokenizer::normal();
    for line_res in buff.lines() {
        let line = line_res?;
        println!("<{}>", line);
        let mut output_line = String::new();
        for token in tokenizer.tokenize(&line) {
            output_line.push_str(token);
            output_line.push_str("---");
        }
        output_line.push_str("\n");
        wtr.write_all(&output_line.as_bytes())?;
    }
    wtr.flush()?;
    Ok(())
}