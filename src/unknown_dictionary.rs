use std::io;
use crate::{CharacterDefinitions, WordEntry};
use std::path::Path;
use std::io::{BufReader, BufRead};
use crate::ParsingError;
use std::str::FromStr;
use crate::character_definition::CategoryId;


//TODO optimize
pub struct UnknownDictionary {
    category_references: Vec<Vec<u32>>,
    costs: Vec<WordEntry>,
    features: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct DictionaryEntry {
    surface: String,
    left_id: u32,
    right_id: u32,
    word_cost: i32,
    part_of_speech: Vec<String>,
    other_features: Vec<String>
}


fn parse_dictionary_entry(fields: &[&str]) -> Result<DictionaryEntry, ParsingError> {
    if fields.len() != 11 {
        return Err(ParsingError::ContentError);
    }
    let surface = fields[0];
    let left_id = u32::from_str(fields[1])?;
    let right_id = u32::from_str(fields[2])?;
    let word_cost = i32::from_str(fields[3])?;
    let part_of_speech: Vec<String> = fields[4..10].iter()
        .cloned()
        .filter(|&pos| pos != "*")
        .map(str::to_string)
        .collect();
    let other_features: Vec<String> = fields[10..].iter()
        .cloned()
        .filter(|&pos| pos != "*")
        .map(str::to_string)
        .collect();
    Ok(DictionaryEntry {
        surface: surface.to_string(),
        left_id,
        right_id,
        word_cost,
        part_of_speech,
        other_features
    })
}



fn make_costs_array(entries: &[DictionaryEntry]) -> Vec<WordEntry> {
    entries
        .iter()
        .map(|e| {
            assert_eq!(e.left_id, e.right_id);
            WordEntry {
                cost_id: e.left_id,
                word_cost: e.word_cost
            }
        })
        .collect()
}


fn get_entry_id_matching_surface(entries: &[DictionaryEntry], target_surface: &str) -> Vec<u32> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(entry_id, entry)|
            if entry.surface == target_surface {
                Some(entry_id as u32)
            } else {
                None
            })
        .collect()
}

fn make_category_references(
    categories: &[String],
    entries: &[DictionaryEntry]) -> Vec<Vec<u32>> {
    categories
        .iter()
        .map(|category| get_entry_id_matching_surface(entries, category))
        .collect()
}

fn make_features(entries: &[DictionaryEntry]) -> Vec<Vec<String>> {
    entries
        .iter()
        .map(|entry| {
            let mut features = entry.part_of_speech.clone();
            features.extend_from_slice(&entry.other_features[..]);
            features
        })
        .collect()
}

impl UnknownDictionary {

    pub fn parse_read<R: io::Read>(
        categories: &[String],
        input_read: R) -> Result<UnknownDictionary, ParsingError> {
        let file = BufReader::new(input_read);
        let mut unknown_dict_entries = Vec::new();
        for line_res in file.lines() {
            let line = line_res?;
            let fields: Vec<&str> = line.split(",").collect::<Vec<&str>>();
            let entry = parse_dictionary_entry(&fields[..])?;
            unknown_dict_entries.push(entry);
        }

        let category_references = make_category_references(categories, &unknown_dict_entries[..]);
        let costs = make_costs_array(&unknown_dict_entries[..]);
        let features = make_features(&unknown_dict_entries[..]);

        Ok(UnknownDictionary {
            category_references,
            costs,
            features,
        })
    }

    pub fn parse(char_definitions: &CharacterDefinitions, dir: &Path) -> Result<UnknownDictionary, ParsingError> {
        let path = dir.join(Path::new("unk.def"));
        let unk_def = crate::read_all(&path)?;
        Self::parse_read(char_definitions.categories(), unk_def.as_bytes())
    }

    pub fn lookup_word_ids(&self, category_id: CategoryId) -> &[u32] {
        &self.category_references[category_id.0][..]
    }


    pub fn load(char_definitions: &CharacterDefinitions) -> Result<UnknownDictionary, ParsingError> {
        Self::parse(char_definitions, crate::ipadic_path())
    }
}

#[cfg(test)]
mod tests {
    use crate::CharacterDefinitions;
    use super::UnknownDictionary;

    #[test]
    fn test_parse_unknown_dictionary() {
        let char_defs = CharacterDefinitions::load();
        let unknown_dict = UnknownDictionary::parse(&char_defs, crate::ipadic_path()).unwrap();
    }
}