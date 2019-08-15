use std::io;
use fst;
use crate::WordEntry;
use fst::raw::Output;

pub struct PrefixDictBuilder<W: io::Write> {
    builder: fst::MapBuilder<W>,
    buffer: Vec<u8>,
}

// TODO remove exposure o fst Error
impl<W: io::Write> PrefixDictBuilder<W> {

    pub(crate) fn new(wtr: W) -> fst::Result<PrefixDictBuilder<W>> {
        Ok(PrefixDictBuilder {
            builder: fst::MapBuilder::new(wtr)?,
            buffer: vec![]
        })
    }

    pub fn insert(&mut self, key: &[u8], word_entries: &[WordEntry]) -> fst::Result<()> {
        if word_entries.is_empty() {
            return Ok(())
        }
        self.buffer.clear();
        self.buffer.extend(key);
        self.buffer.push(0u8);
        for (b, &word_entry) in word_entries.iter().clone().enumerate() {
            self.buffer.resize(key.len() + 1, 0u8);
            self.buffer.push(b as u8);
            self.builder.insert(&self.buffer[..], word_entry.encode_as_u64())?;
        }
        Ok(())
    }

    pub fn finish(self) -> fst::Result<()> {
        self.builder.finish()
    }
}

pub struct PrefixDict {
    pub fst: fst::raw::Fst
}

impl PrefixDict {

    pub fn from_static_slice(bytes: &'static [u8]) -> fst::Result<PrefixDict> {
        let fst = fst::raw::Fst::from_static_slice(bytes)?;
        Ok(PrefixDict {
            fst
        })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> fst::Result<PrefixDict> {
        let fst = fst::raw::Fst::from_bytes(bytes)?;
        Ok(PrefixDict {
            fst
        })
    }

    pub fn prefix<'a>(&'a self, s: &'a str) -> impl Iterator<Item=(usize, WordEntry)> + 'a {
        s.as_bytes()
         .iter()
         .scan((self.fst.root(), Output::zero()), move |(node, output), &byte, | {
            if let Some(b_index) = node.find_input(byte) {
                let transition = node.transition(b_index);
                *output = output.cat(transition.out);
                *node = self.fst.node(transition.addr);
                return Some((*node, *output));
            }
            None
         })
         .enumerate()
         .flat_map(move|(prefix_len, (node, out))|
             node.find_input(0u8)
                 .map(|b_index| {
                     let t = node.transition(b_index);
                     (prefix_len + 1, self.fst.node(t.addr), out.cat(t.out))
                 })
         )
         .flat_map(|(prefix_len, node, out)|
            (0..)
                .map(move|b|
                    node.find_input(b)
                        .map(|b_index| {
                            let t = node.transition(b_index);
                            let out = out.cat(t.out);
                            (prefix_len,  WordEntry::decode_from_u64(out.value()))
                        })
                )
                .take_while(Option::is_some)
                .flatten()
         )
    }
}



#[cfg(test)]
mod tests {
    use crate::WordEntry;
    use crate::prefix_dict::PrefixDictBuilder;
    use super::PrefixDict;

    #[test]
    fn test_fst_prefix() {
        let mut buffer = Vec::new();
        let mut builder =
            PrefixDictBuilder::new(&mut buffer).unwrap();
        builder.insert(b"aaa", &[
            WordEntry { word_cost: 0, cost_id: 1 },
            WordEntry {word_cost: 0, cost_id: 2 }
        ]).unwrap();
        builder.insert(b"aaab", &[
            WordEntry { word_cost: 0, cost_id: 3 }
        ]).unwrap();
        builder.finish().unwrap();
        let fst = PrefixDict::from_bytes(buffer).unwrap();
        assert_eq!(fst.prefix("aaabc").collect::<Vec<_>>(),
            vec![(3, WordEntry { word_cost: 0, cost_id: 1 }),
                  (3, WordEntry { word_cost: 0, cost_id: 2 }),
                  (4, WordEntry {word_cost: 0, cost_id: 3 })]
        );
        assert_eq!(fst.prefix("aaac").collect::<Vec<_>>(),
                   vec![(3, WordEntry { word_cost: 0, cost_id: 1 }),
                        (3, WordEntry { word_cost: 0, cost_id: 2 })]
        );
    }
}
