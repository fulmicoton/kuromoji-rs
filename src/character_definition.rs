use serde::{Deserialize, Serialize};

const CHAR_DEFINITION_DATA: &'static [u8] = include_bytes!("../dict/char_def.bin");

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct CategoryData {
    pub invoke: bool,
    pub group: bool,
    pub length: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Copy, PartialOrd, Ord, Eq, PartialEq)]
pub struct CategoryId(pub usize);

#[derive(Serialize, Deserialize)]
pub struct CharacterDefinitions {
    pub category_definitions: Vec<CategoryData>,
    pub category_names: Vec<String>,
    pub mapping: Vec<(u32, u32, Vec<CategoryId>)>,
    pub default_category: [CategoryId; 1],
}

impl CharacterDefinitions {
    pub fn categories(&self) -> &[String] {
        &self.category_names[..]
    }

    pub fn load() -> CharacterDefinitions {
        bincode::deserialize(CHAR_DEFINITION_DATA)
            .expect("Failed to deserialize char definition data")
    }
    pub fn lookup_definition(&self, category_id: CategoryId) -> &CategoryData {
        &self.category_definitions[category_id.0]
    }

    pub fn category_name(&self, category_id: CategoryId) -> &str {
        &self.category_names[category_id.0 as usize]
    }

    pub fn lookup_categories(&self, c: char, categories_buffer: &mut Vec<CategoryId>) {
        // TODO optimize
        categories_buffer.clear();
        //let mut res = &self.default_category[..];
        let c = c as u32;
        for (start, stop, category_ids) in &self.mapping {
            if *start <= c && *stop >= c {
                for cat in category_ids {
                    if !categories_buffer.contains(cat) {
                        categories_buffer.push(*cat);
                    }
                }
            }
        }
        if categories_buffer.is_empty() {
            categories_buffer.extend(&self.default_category[..]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisa() {
        let mut v = vec![];
        let char_definitions = CharacterDefinitions::load();
        char_definitions.lookup_categories('々', &mut v);
        let category_ids: Vec<&str> = v
            .iter()
            .map(|&category_id| char_definitions.category_name(category_id))
            .collect();
        assert_eq!(category_ids, &["KANJI", "SYMBOL"]);
    }

    #[test]
    fn test_jp_hyphen() {
        let mut v = vec![];
        let char_definitions = CharacterDefinitions::load();
        char_definitions.lookup_categories('ー', &mut v);
        let category_ids: Vec<&str> = v
            .iter()
            .map(|&category_id| char_definitions.category_name(category_id))
            .collect();
        assert_eq!(category_ids, &["KATAKANA"]);
    }

    #[test]
    fn test_char_definitions() {
        let mut v = vec![];
        let char_definitions = CharacterDefinitions::load();
        {
            char_definitions.lookup_categories('あ', &mut v);
            assert_eq!(v.len(), 1);
            assert_eq!(char_definitions.category_name(v[0]), "HIRAGANA");
        }
        {
            char_definitions.lookup_categories('@', &mut v);
            assert_eq!(v.len(), 1);
            assert_eq!(char_definitions.category_name(v[0]), "SYMBOL");
        }
        {
            char_definitions.lookup_categories('一', &mut v);
            assert_eq!(v.len(), 2);
            assert_eq!(char_definitions.category_name(v[0]), "KANJI");
            assert_eq!(char_definitions.category_name(v[1]), "KANJINUMERIC");
        }
    }

}
