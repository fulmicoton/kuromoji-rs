use std::io;
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use std::fmt::Debug;
use encoding::all::UTF_16LE;
use encoding::{Encoding, DecoderTrap};
use byteorder::{ByteOrder, LittleEndian};

const DEFAULT_CATEGORY_NAME: &'static str = "DEFAULT";

#[derive(Debug)]
pub enum ParsingError {
    Io(io::Error),
    ParsingError(String)
}

impl ParsingError {
    fn from_error<D: Debug>(error: D) -> ParsingError {
        ParsingError::ParsingError(format!("{:?}", error))
    }
}

impl From<io::Error> for ParsingError {
    fn from(io_err: io::Error) -> Self {
        ParsingError::Io(io_err)
    }
}

#[derive(Default)]
pub struct CharacterDefinitionsBuilder {
    category_definition: Vec<CategoryData>,
    category_index: HashMap<String, CategoryId>,
    char_ranges: Vec<(u32, u32, Vec<CategoryId>)>
}


pub struct CategoryData {
    pub invoke: bool,
    pub group:  bool,
    pub length: u32
}

fn ucs2_to_unicode(ucs2_codepoint: u16) -> u32 {
    let mut buf = [0u8; 2];
    LittleEndian::write_u16(&mut buf[..], ucs2_codepoint);
    let s: String = UTF_16LE.decode(&buf[..], DecoderTrap::Strict).unwrap();
    let chrs: Vec<char> = s.chars().collect();
    assert_eq!(chrs.len(), 1);
    chrs[0] as u32
}

fn parse_hex_codepoint(s: &str) -> Result<u32, ParsingError> {
    let removed_0x = s.trim_start_matches("0x");
    let ucs2_codepoint = u16::from_str_radix(removed_0x, 16).map_err(ParsingError::from_error)?;
    let utf8_str = ucs2_to_unicode(ucs2_codepoint);
    Ok(utf8_str)
}

#[derive(Clone, Debug, Hash, Copy, PartialOrd, Ord, Eq, PartialEq)]
pub struct CategoryId(pub usize);

impl CharacterDefinitionsBuilder {

    pub fn category_id(&mut self, category_name: &str) -> CategoryId {
        let num_categories = self.category_index.len();
        *self.category_index
            .entry(category_name.to_string())
            .or_insert(CategoryId(num_categories))
    }

    pub fn parse_read<R: io::Read>(&mut self, input_read: R) -> Result<(), ParsingError> {
        let file = BufReader::new(input_read);
        for line_res in file.lines() {
            let line = line_res?;
            let line_str = line.split('#').next().unwrap().trim();
            if line_str.is_empty() {
                continue;
            }
            if line_str.starts_with("0x") {
                self.parse_range(line_str)?;
            } else {
                self.parse_category(line_str)?;
            }
        }
        Ok(())
    }



    fn parse_range(&mut self, line: &str) -> Result<(), ParsingError> {
        let fields: Vec<&str> =  line.split_whitespace().collect();
        let range_bounds: Vec<&str> = fields[0].split("..").collect();
        let lower_bound: u32;
        let higher_bound: u32   ;
        match range_bounds.len() {
            1 => {
                lower_bound = parse_hex_codepoint(range_bounds[0])?;
                higher_bound = lower_bound;
            }
            2 => {
                lower_bound = parse_hex_codepoint(range_bounds[0])?;
                // the right bound is included in the file.
                higher_bound = parse_hex_codepoint(range_bounds[1])?;
            }
            _ => {
                return Err(ParsingError::ParsingError(format!("Invalid line: {}", line)));
            }
        }
        let category_ids: Vec<CategoryId> = fields[1..]
            .iter()
            .map(|category| self.category_id(category))
            .collect();
        let ranges = self.char_ranges.push((lower_bound, higher_bound, category_ids));
        Ok(())
    }

    fn parse_category(&mut self, line: &str) -> Result<(), ParsingError> {
        let fields = line.split_ascii_whitespace().collect::<Vec<&str>>();
        if fields.len() != 4 {
            return Err(ParsingError::ParsingError(format!("Expected 4 fields. Got {} in {}", fields.len(), line)));
        }
        let invoke = fields[1].parse::<u32>().map_err(ParsingError::from_error)? == 1;
        let group = fields[2].parse::<u32>().map_err(ParsingError::from_error)? == 1;
        let length = fields[3].parse::<u32>().map_err(ParsingError::from_error)?;
        let category_data = CategoryData { invoke, group, length };
        let category_id = self.category_id(fields[0]);
        self.category_definition.push(category_data);
        Ok(())
    }

    pub fn build(self) -> CharacterDefinitions {
        let mut category_names: Vec<String> = (0..self.category_index.len())
            .map(|_| String::new())
            .collect();
        for (category_name, category_id) in &self.category_index {
            category_names[category_id.0] = category_name.clone();
        }
        let default_category = *self.category_index.get(DEFAULT_CATEGORY_NAME)
            .expect("No default category defined.");
        CharacterDefinitions {
            category_definitions: self.category_definition,
            category_names,
            mapping: self.char_ranges,
            default_category: [default_category]
        }
    }
}

pub struct CharacterDefinitions {
    category_definitions: Vec<CategoryData>,
    category_names: Vec<String>,
    mapping: Vec<(u32, u32, Vec<CategoryId>)>,
    default_category: [CategoryId; 1]
}

impl CharacterDefinitions {

    pub fn categories(&self) -> &[String] {
        &self.category_names[..]
    }

    pub fn parse(dir: &Path) -> Result<CharacterDefinitions, ParsingError> {
        let mut char_definitions_builder = CharacterDefinitionsBuilder::default();
        let path = dir.join(Path::new("char.def"));
        let input_read = File::open(path)?;
        char_definitions_builder.parse_read(input_read)?;
        Ok(char_definitions_builder.build())
    }

    pub fn load() -> CharacterDefinitions {
        Self::parse(crate::ipadic_path()).unwrap()
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
    fn test_lookup() {
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

/*
public static final int INVOKE = 0;
public static final int GROUP = 1;

private static final String DEFAULT_CATEGORY = "DEFAULT";

private static final int LENGTH = 2; // Not used as of now

private final int[][] categoryDefinitions;

private final int[][] codepointMappings;

private final String[] categorySymbols;

private final int[] defaultCategory;

public CharacterDefinitions(int[][] categoryDefinitions,
int[][] codepointMappings,
String[] categorySymbols) {
this.categoryDefinitions = categoryDefinitions;
this.codepointMappings = codepointMappings;
this.categorySymbols = categorySymbols;
this.defaultCategory = lookupCategories(new String[]{DEFAULT_CATEGORY});
}

public int[] lookupCategories(char c) {
int[] mappings = codepointMappings[c];

if (mappings == null) {
return defaultCategory;
}

return mappings;
}

public int[] lookupDefinition(int category) {
return categoryDefinitions[category];
}

public static CharacterDefinitions newInstance(ResourceResolver resolver) throws IOException {
InputStream charDefInput = resolver.resolve(CHARACTER_DEFINITIONS_FILENAME);

int[][] definitions = IntegerArrayIO.readSparseArray2D(charDefInput);
int[][] mappings = IntegerArrayIO.readSparseArray2D(charDefInput);
String[] symbols = StringArrayIO.readArray(charDefInput);

CharacterDefinitions characterDefinition = new CharacterDefinitions(
definitions,
mappings,
symbols
);

return characterDefinition;
}

public void setCategories(char c, String[] categoryNames) {
codepointMappings[c] = lookupCategories(categoryNames);
}

private int[] lookupCategories(String[] categoryNames) {
int[] categories = new int[categoryNames.length];

for (int i = 0; i < categoryNames.length; i++) {
String category = categoryNames[i];
int categoryIndex = -1;

for (int j = 0; j < categorySymbols.length; j++) {
if (category.equals(categorySymbols[j])) {
categoryIndex = j;
}
}

if (categoryIndex < 0) {
throw new RuntimeException("No category '" + category + "' found");
}

categories[i] = categoryIndex;
}

return categories;
}
}
*/
