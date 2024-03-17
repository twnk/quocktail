use lazy_static::lazy_static;
use pulldown_cmark::{
    CowStr, Event, MetadataBlockKind, Options, Parser, Tag, TagEnd, TextMergeStream,
};
use yaml_rust::{yaml, Yaml, YamlLoader};

#[derive(Debug)]
pub struct Filters(Vec<(Yaml, Option<String>)>);

impl Filters {
    pub fn new(filters: Vec<(String, Option<String>)>) -> Filters {
        let yamlified = filters
            .into_iter()
            .map(|(k, v)| (Yaml::String(k), v))
            .collect();
        Filters(yamlified)
    }
}

impl IntoIterator for Filters {
    type Item = (Yaml, Option<String>);

    type IntoIter = <Vec<(Yaml, Option<std::string::String>)> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Filters {
    type Item = &'a (Yaml, Option<String>);

    type IntoIter = <&'a Vec<(Yaml, Option<std::string::String>)> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct Ingredients {
    pub hash: yaml::Hash,
}

lazy_static! {
    pub static ref YAML_OR_KEY: Yaml = Yaml::String("or".to_string());
}

impl Ingredients {
    fn new(str: CowStr<'_>) -> Option<Ingredients> {
        let mut frontmatter_vec = match YamlLoader::load_from_str(&str) {
            Ok(yaml) => yaml,
            Err(e) => {
                println!("{:?}", e);
                return None;
            }
        };

        let first_item = frontmatter_vec.pop()?;
        let hash = first_item.into_hash()?;

        Some(Ingredients { hash })
    }

    fn filter(&self, inclusion_filters: &Filters, exclusion_filters: &Filters) -> bool {
        for (exclude_key, maybe_exclude_val) in exclusion_filters {
            let val = match self.hash.get(exclude_key) {
                Some(v) => v,     // key matches, require further checks
                None => continue, // no matching key, don't exclude
            };

            let exclude_val = if let Some(ex_val) = maybe_exclude_val {
                ex_val
            } else if !matches!(val, Yaml::Null) {
                return false;
            } else {
                continue;
            };

            match val {
                // a single value. this must not match.
                Yaml::String(s) if exclude_val.eq(s) => return false,
                // a list of values. must not match any.
                Yaml::Array(arr) => {
                    for s in arr.iter() {
                        if let Yaml::String(s) = s {
                            if exclude_val.eq(s) {
                                return false;
                            }
                        }
                    }
                }
                // an or'd list.
                Yaml::Hash(_hash) => {}
                // a null value, don't exclude
                Yaml::Null => continue,
                _ => {}
            }

            // note that val can be e.g. an array or another hash!
            // but in this case, this is only used for or, e.g. "vodka or gin".
            // we don't want to exclude based on or'd ingredients... unless... they all match! :grimace:
            // this is not handled yet. one thing at a time.
        }

        'outer: for (include_key, maybe_include_val) in inclusion_filters {
            let val = match self.hash.get(include_key) {
                Some(v) => v,
                None => return false, // key must be included
            };

            let include_val = if let Some(in_val) = maybe_include_val {
                in_val
            } else {
                // we only require a key match
                if matches!(val, Yaml::Null) {
                    // null means key doesn't match
                    return false;
                } else {
                    continue;
                }
            };

            match val {
                // a single value - must match
                Yaml::String(s) if include_val.ne(s) => return false,
                // an array of required values - one must match
                Yaml::Array(arr) => {
                    for s in arr.iter() {
                        match s {
                            Yaml::String(s) if include_val.eq(s) => continue 'outer,
                            _ => {}
                        }
                    }
                    return false;
                }
                // an array of optional ("or: - a - b") values - one must match
                Yaml::Hash(hash) => {
                    if let Some(Yaml::Array(or_arr)) = hash.get(&YAML_OR_KEY) {
                        for s in or_arr.iter() {
                            match s {
                                Yaml::String(s) if include_val.eq(s) => continue 'outer,
                                _ => {}
                            }
                        }
                        return false;
                    }
                }
                // null value, doesn't match
                Yaml::Null => return false,
                _ => {}
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct Recipe {
    /// by convention, Title Case name
    name: String,
    ingredients: Ingredients,
    instructions: String,
}

lazy_static! {
    static ref PARSER_OPTIONS: Options = {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        options
    };
}

impl Recipe {
    pub fn filtered_new(
        input: &str,
        name: String,
        inclusion_filters: &Filters,
        exclusion_filters: &Filters,
    ) -> Option<Self> {
        let parser = Parser::new_ext(input, *PARSER_OPTIONS);

        let mut iterator = TextMergeStream::new(parser).peekable();

        if !matches!(
            iterator.next(),
            Some(Event::Start(Tag::MetadataBlock(
                MetadataBlockKind::YamlStyle
            )))
        ) {
            return None;
        }

        let ingredients = if let Some(Event::Text(str)) = iterator.next() {
            let ingredients = Ingredients::new(str)?;
            if ingredients.filter(inclusion_filters, exclusion_filters) {
                ingredients
            } else {
                return None;
            }
        } else {
            return None;
        };

        if !matches!(
            iterator.next(),
            Some(Event::End(TagEnd::MetadataBlock(
                MetadataBlockKind::YamlStyle
            )))
        ) {
            return None;
        }
        if !matches!(iterator.next(), Some(Event::Start(Tag::Paragraph))) {
            return None;
        }

        // most recipes are around 100 chars long
        let mut instructions = String::with_capacity(200);

        for event in iterator {
            match event {
                Event::Text(s) => instructions.push_str(&s),
                Event::SoftBreak => instructions.push('\n'),
                Event::HardBreak => instructions.push_str("\n\n"),
                _ => {}
            }
        }

        Some(Recipe {
            name,
            ingredients,
            instructions,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ingredients(&self) -> &Ingredients {
        &self.ingredients
    }
    pub fn instructions(&self) -> &String {
        &self.instructions
    }
}
