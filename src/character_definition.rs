use serde::{Serialize, Deserialize};

const CHAR_DEFINITION_DATA: &'static [u8] = include_bytes!("../dict/char_def.bin");

#[derive(Serialize, Deserialize)]
pub struct CategoryData {
    pub invoke: bool,
    pub group:  bool,
    pub length: u32
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Copy, PartialOrd, Ord, Eq, PartialEq)]
pub struct CategoryId(pub usize);

#[derive(Serialize, Deserialize)]
pub struct CharacterDefinitions {
    pub category_definitions: Vec<CategoryData>,
    pub category_names: Vec<String>,
    pub mapping: Vec<(u32, u32, Vec<CategoryId>)>,
    pub default_category: [CategoryId; 1]
}

impl CharacterDefinitions {

    pub fn categories(&self) -> &[String] {
        &self.category_names[..]
    }

    pub fn load() -> CharacterDefinitions {
        bincode::deserialize(CHAR_DEFINITION_DATA).expect("Failed to deserialize char definition data")
    }
    pub fn lookup_definition(&self, category_id: CategoryId) -> &CategoryData {
        &self.category_definitions[category_id.0]
    }

    pub fn category_name(&self, category_id: CategoryId) -> &str {
        &self.category_names[category_id.0 as usize]
    }

    pub fn lookup_categories(&self, c: char) -> &[CategoryId] {
        let mut res = &self.default_category[..];
        let c = c as u32;
        for (start, stop, category_ids) in &self.mapping {
            if *start <= c && *stop >= c {
                res = &category_ids[..];
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_definitions() {
        let char_definitions = CharacterDefinitions::load();
        {
            let categories = char_definitions.lookup_categories('あ');
            assert_eq!(categories.len(), 1);
            assert_eq!(char_definitions.category_name(categories[0]), "HIRAGANA");
        }
        {
            let categories = char_definitions.lookup_categories('@');
            assert_eq!(categories.len(), 1);
            assert_eq!(char_definitions.category_name(categories[0]), "SYMBOL");
        }
        {
            let categories = char_definitions.lookup_categories('一');
            assert_eq!(categories.len(), 2);
            assert_eq!(char_definitions.category_name(categories[0]), "KANJINUMERIC");
            assert_eq!(char_definitions.category_name(categories[1]), "KANJI");

        }
    }

}

