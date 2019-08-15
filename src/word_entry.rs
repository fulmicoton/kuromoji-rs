#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct WordEntry {
    pub word_cost: i32,
    pub cost_id: u32,
}

impl WordEntry {
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
}
