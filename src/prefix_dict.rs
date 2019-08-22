use tantivy_fst;
use crate::WordEntry;
use tantivy_fst::raw::Output;
use std::ops::Deref;

const IPAD_DATA: &'static [u8] = include_bytes!("../dict/dict.fst");
const IPAD_VALS: &'static [u8] = include_bytes!("../dict/dict.vals");

pub struct PrefixDict<Data=&'static [u8 ]> {
    pub fst: tantivy_fst::raw::Fst<Data>,
    vals_data: Data
}


impl Default for PrefixDict<&'static [u8]> {
    fn default() -> PrefixDict<&'static [u8]>  {
        PrefixDict::from_static_slice(IPAD_DATA, IPAD_VALS).unwrap()
    }
}

impl PrefixDict<&'static [u8]> {
    pub fn from_static_slice(fst_data: &'static [u8], vals_data: &'static [u8]) -> tantivy_fst::Result<PrefixDict> {
        let fst = tantivy_fst::raw::Fst::new(fst_data)?;
        Ok(PrefixDict {
            fst,
            vals_data
        })
    }
}


impl<D: Deref<Target=[u8]>> PrefixDict<D> {

    pub fn prefix<'a>(&'a self, s: &'a str) -> impl Iterator<Item=(usize, WordEntry)> + 'a {
        s.as_bytes()
         .iter()
         .scan((0, self.fst.root(), Output::zero()),
                move |(prefix_len, node, output), &byte, | {
            if let Some(b_index) = node.find_input(byte) {
                let transition = node.transition(b_index);
                *prefix_len += 1;
                *output = output.cat(transition.out);
                *node = self.fst.node(transition.addr);
                return Some((node.is_final(), *prefix_len, output.value()));
            }
            None
         })
         .filter_map(|(is_final, prefix_len, offset_len)|
             if is_final {
                Some((prefix_len, offset_len))
             } else {
                 None
             })
         .flat_map(move|(prefix_len, offset_len)| {
            let len = offset_len & ((1u64 << 5) - 1u64);
            let offset = offset_len >> 5u64;
            let offset_bytes = (offset as usize) * WordEntry::SERIALIZED_LEN;
            let data: &[u8] = &self.vals_data[offset_bytes..];
            (0..len as usize).map(move |i|
                (prefix_len, WordEntry::deserialize(&data[WordEntry::SERIALIZED_LEN * i..])))
         })
    }

}



#[cfg(test)]
mod tests {
    use super::PrefixDict;

    #[test]
    fn test_fst_prefix() {
        let prefix_dict = PrefixDict::default();
        for (a, word_entry) in prefix_dict.prefix("下北沢") {
            println!("{} {:?}", a, word_entry)
        }
    }

    /*
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
        let fst = PrefixDict:(buffer).unwrap();
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
    */
}
