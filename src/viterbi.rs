use std::u32;
use connection::ConnectionCostMatrix;
use fst::raw::Output;
use fst;
use WordEntry;

const EOS_NODE: NodeId = NodeId(1u32);

#[derive(Clone, Copy, Debug)]
pub enum NodeType {
    KNOWN,
    UNKNOWN,
    USER,
    INSERTED
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::KNOWN
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct NodeId(pub u32);

#[derive(Default, Clone, Debug)]
pub struct Node {
    pub node_type: NodeType,
    // pub word_id: u32, //< word id in the dictionary

    pub word_entry: WordEntry,

    pub path_cost: i32,
    pub left_node: Option<NodeId>,

    pub start_index: u32,
    pub stop_index: u32,
}



#[derive(Default)]
pub struct Lattice {
    capacity: usize,
    nodes: Vec<Node>,
    starts_at: Vec<Vec<NodeId>>,
    ends_at: Vec<Vec<NodeId>>,
}

impl Lattice {

    pub fn clear(&mut self) {
        for node_vec in &mut self.starts_at {
            node_vec.clear();
        }
        for node_vec in &mut self.ends_at {
            node_vec.clear();
        }
        self.nodes.clear()
    }

    #[inline(never)]
    pub fn set_text(&mut self, dict: &fst::raw::Fst, text: &str) {
        let text: &[u8] = text.as_bytes();
        let len = text.len();
        if self.capacity < text.len() {
            self.capacity = text.len();
            self.nodes.clear();
            self.starts_at = vec![Vec::new(); len + 1];
            self.ends_at = vec![Vec::new(); len + 1];
        } else {
            self.clear();
        }
        let bos_node_id = self.add_node(Node::default());
        let eos_node_id = self.add_node(Node::default());
        assert_eq!(EOS_NODE, eos_node_id);
        self.ends_at[0].push(bos_node_id);
        self.starts_at[len].push(eos_node_id);

        let mut unknown_word_end_index: Option<usize> = None;

        for start in 0..len - 1 {
            if self.ends_at[start as usize].is_empty() {
                continue;
            }
            let mut node = dict.root();
            let mut out = Output::zero();
            let suffix = &text[start..];
            for (i, b) in suffix.iter().cloned().enumerate() {
                node = match node.find_input(b) {
                    None => {
                        break;
                    },
                    Some(b_index) => {
                        let t = node.transition(b_index);
                        out = out.cat(t.out);
                        dict.node(t.addr)
                    }
                };
                {
                    if let Some(b_index) = node.find_input(0u8) {
                        let t = node.transition(b_index);
                        let children_out = out.cat(t.out);
                        let leaves_node = dict.node(t.addr);
                        for b in 0.. {
                            if let Some(leaf_transition_index) = leaves_node.find_input(b) {
                                let leaf_transition = leaves_node.transition(leaf_transition_index);
                                assert!(dict.node(leaf_transition.addr).is_final());
                                let dest_out = children_out.cat(leaf_transition.out);
                                let word_entry = WordEntry::decode_from_u64(dest_out.value());
                                let node = Node {
                                    node_type: NodeType::KNOWN,
                                    word_entry: word_entry,
                                    left_node: None,
                                    start_index: start as u32,
                                    stop_index: (start + i + 1) as u32,
                                    path_cost: i32::max_value(),
                                };
                                self.add_node_in_lattice(node);
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            /*
            if unknown_word_end_index <= start {
                let mut unknown_word_length = 0;
                let first_char: char = suffix.chars().next().unwrap();
                let is_invoke = self.character_definition.is_invoke(firstCharacter);
                if (isInvoke) {
                    unknownWordLength = this.unkDictionary.lookup(suffix);
                } else if (!found) {
                    unknownWordLength = this.unkDictionary.lookup(suffix);
                }

                if (unknownWordLength > 0) {
                    String unkWord = suffix.substring(0, unknownWordLength);
                    characterId = this.characterDefinition.lookup(firstCharacter);
                    int[] wordIds = this.unkDictionary.lookupWordIds(characterId);
                    int[] arr$ = wordIds;
                    int len$ = wordIds.length;

                    for(int i$ = 0; i$ < len$; ++i$) {
                    int wordId = arr$[i$];
                    ViterbiNode node = new ViterbiNode(wordId, unkWord, this.unkDictionary.getLeftId(wordId), this.unkDictionary.getRightId(wordId), this.unkDictionary.getWordCost(wordId), startIndex, Type.UNKNOWN);
                    this.addToArrays(node, startIndex + 1, startIndex + 1 + unknownWordLength, startIndexArr, endIndexArr, startSizeArr, endSizeArr);
                    }

                    unknownWordEndIndex = startIndex + unknownWordLength;
                }
            }
            */
        }
    }


    fn add_node_in_lattice(&mut self, node: Node) {
        let start_index = node.start_index as usize;
        let stop_index = node.stop_index as usize;
        let node_id = self.add_node(node);
        self.starts_at[start_index].push(node_id);
        self.ends_at[stop_index].push(node_id);
    }


    fn add_node(&mut self, node: Node) -> NodeId {
        let node_id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        node_id
    }

    pub fn node(&self, node_id: NodeId) -> &Node {
        &self.nodes[node_id.0 as usize]
    }

    pub fn node_mut(&mut self, node_id: NodeId) -> &mut Node {
        &mut self.nodes[node_id.0 as usize]
    }

    #[inline(never)]
    pub fn calculate_path_costs(&mut self, cost_matrix: &ConnectionCostMatrix) {
        let text_len = self.starts_at.len();
        for i in 0..text_len {
            let left_node_ids = &self.ends_at[i];
//
//            if (mode == TokenizerBase.Mode.SEARCH || mode == TokenizerBase.Mode.EXTENDED) {
//                let penalty_cost = getPenaltyCost(node);
//            }
//
            let right_node_ids = &self.starts_at[i];
            for &right_node_id in right_node_ids {
                let right_word_entry = self.node(right_node_id).word_entry;
                let best_path = left_node_ids
                    .iter()
                    .cloned()
                    .map(|left_node_id| {
                        let left_node = self.node(left_node_id);
                        let path_cost = left_node.path_cost +
                            cost_matrix.cost(left_node.word_entry.right_id(), right_word_entry.left_id());
                        (path_cost, left_node_id)
                    })
                    .min_by_key(|&(cost, _)| cost);
                if let Some((best_cost, best_left)) = best_path {
                    let node = &mut self.nodes[right_node_id.0 as usize];
                    node.left_node = Some(best_left);
                    node.path_cost = right_word_entry.word_cost + best_cost;
                }
            }
        }
    }

    pub fn tokens_offset(&self) -> Vec<usize> {
        let mut tokens = vec!();
        let mut node_id = EOS_NODE;
        loop {
            let node = self.node(node_id);
            if let Some(left_node_id) = node.left_node {
                tokens.push(node.start_index as usize);
                node_id = left_node_id;
            } else {
                break;
            }
        }
        tokens.reverse();
        tokens.pop();
        tokens
    }
}


#[cfg(test)]
mod tests {

    use super::WordEntry;

    #[test]
    fn test_wordentry() {
        fn test_serdeser(word_cost: i32, cost_id: u32) {
            let word_entry = WordEntry { word_cost: word_cost, cost_id: cost_id};
                assert_eq!(word_entry, WordEntry::from(word_entry.to()));
        }
        test_serdeser(-1i32, 0u32);
        test_serdeser(-1i32, 1u32);
        // test_serdeser(-1000000000i32, 3000000000u32);
    }


    #[test]
    fn test_fst() {
        use fst::{self, IntoStreamer, Streamer, Map, MapBuilder};

        let mut buffer = Vec::new();
        {
            let mut fst_builder = MapBuilder::new(&mut buffer).unwrap();
            fst_builder.insert("a".as_bytes(), 1);
            fst_builder.insert("a".as_bytes(), 2);
            fst_builder.finish().unwrap();
        }
        let map = Map::from_bytes(buffer).unwrap();
        let mut stream = map.stream();
        while let Some(kv) = stream.next() {
            println!("{:?}", kv);
        }
    }
}