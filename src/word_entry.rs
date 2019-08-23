use std::io;
use byteorder::{ByteOrder, LittleEndian};
use byteorder::WriteBytesExt;
use serde::{Serialize, Deserialize};

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WordEntry {
    pub word_cost: i32,
    pub cost_id: u32,
}

impl WordEntry {

    pub const SERIALIZED_LEN: usize = 4;

    pub fn decode_from_u64(encoded_value: u64) -> WordEntry {
        WordEntry {
            word_cost: (encoded_value & <u32>::max_value() as u64) as i32,
            cost_id: (encoded_value >> 32u64) as u32,
        }
    }

    pub fn encode_as_u64(&self) -> u64 {
        let cost = self.cost_id as u64;
        let cost_shifted: u64 = cost << 32u64;
        let word_cost_cast: u64 = (self.word_cost as u32) as u64;
        cost_shifted + word_cost_cast
    }

    pub fn left_id(&self) -> u32 {
        self.cost_id
    }

    pub fn right_id(&self) -> u32 {
        self.cost_id
    }

    pub fn serialize<W: io::Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write_i16::<LittleEndian>(self.word_cost as i16)?;
        wtr.write_u16::<LittleEndian>(self.cost_id as u16)?;
        Ok(())
    }


    pub fn deserialize(data: &[u8]) -> WordEntry {
        let word_cost = LittleEndian::read_i16(&data[0..2]);
        let cost_id = LittleEndian::read_u16(&data[2..4]);
        WordEntry {
            word_cost: word_cost as i32,
            cost_id: cost_id as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::WordEntry;

    #[test]
    fn test_word_entry() {
        let mut buffer = Vec::new();
        let word_entry = WordEntry {
            word_cost: -17i32,
            cost_id: 1411u32
        };
        word_entry.serialize(&mut buffer).unwrap();
        assert_eq!(WordEntry::SERIALIZED_LEN, buffer.len());
        let word_entry2 = WordEntry::deserialize(&buffer[..]);
        assert_eq!(word_entry, word_entry2);
    }
}