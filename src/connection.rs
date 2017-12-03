use std::io::{self, BufReader};
use std::fs::File;
use std::io::BufRead;
use std::str::FromStr;

pub struct ConnectionCostMatrix {
    costs: Vec<i32>,
    backward_size: u32,
}

impl ConnectionCostMatrix {
    pub fn load(path: &str) -> io::Result<ConnectionCostMatrix> {
        let reader = File::open(path)?;
        let buf_reader = BufReader::new(reader);
        let mut lines = buf_reader.lines().map(|line_res| line_res
            .unwrap()
            .split(' ')
            .map(|s| i32::from_str(s).unwrap())
            .collect::<Vec<i32>>());
        let header = lines.next().unwrap();
        let forward_size = header[0] as u32;
        let backward_size = header[0] as u32;
        let len = (forward_size * backward_size) as usize;
        let mut costs = vec![i32::max_value(); len];
        for fields in lines {
            let forward_id = fields[0] as u32;
            let backward_id = fields[1] as u32;
            let cost = fields[2];
            costs[(backward_id + forward_id * backward_size) as usize] = cost;
        }
        Ok(ConnectionCostMatrix {
            costs: costs,
            backward_size: backward_size as u32,
        })
    }

    pub fn cost(&self, backward_id: u32, forward_id: u32) -> i32 {
        let cost_id = backward_id + forward_id * self.backward_size;
        self.costs[cost_id as usize]
    }
}
