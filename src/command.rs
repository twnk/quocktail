//! Contains command parsers and logic.

use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, ValueEnum};
use convert_case::{Case, Casing};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::fs::File;
use std::io::Read;
use supports_color::Stream;
use walkdir::WalkDir;

use crate::output::Recipe;

#[derive(Debug, Parser)]
pub struct Quocktail {
    /// Should the output be colored?
    #[clap(long, value_enum, global = true, default_value = "auto")]
    color: Color,

    /// Queries in the form `key=value`, or just `key`
    /// e.g. `-Q garnish` matches anything with a garnish
    /// e.g. `-Q base=vodka` matches anything with a vodka base
    #[clap(long, short='q', value_parser = parse_key_some_val)]
    query: Vec<(String, Option<String>)>,

    /// Negative queries in the form `key=value`, or just `key`
    /// e.g. `-N garnish` excludes anything with a garnish
    /// e.g. `-N base=vodka` excludes anything with a vodka base
    #[clap(long, short='n', value_parser = parse_key_some_val)]
    exclude: Vec<(String, Option<String>)>,

    /// If specified, limit the output to this number of recipes, randomly chosen
    #[clap(long, short = 'c')]
    count: Option<usize>,

    path: Utf8PathBuf,
}

fn parse_key_some_val(s: &str) -> color_eyre::Result<(String, Option<String>)> {
    if let Some((k, v)) = s.split_once('=') {
        Ok((k.to_string(), Some(v.to_string())))
    } else {
        Ok((s.to_string(), None))
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Color {
    Always,
    Auto,
    Never,
}

impl Color {
    fn supports_color_on(self, stream: supports_color::Stream) -> bool {
        match self {
            Color::Always => true,
            Color::Auto => supports_color::on_cached(stream).is_some(),
            Color::Never => false,
        }
    }
}

impl Quocktail {
    pub fn exec(self) -> color_eyre::Result<()> {
        let inclusion_filters = self.query;
        let exclusion_filters = self.exclude;

        let matter = Matter::<YAML>::new();

        let pods = WalkDir::new(self.path)
            .into_iter()
            .filter_map(|entry| {
                let entry = if let Ok(entry) = entry {
                    entry
                } else {
                    return None;
                };

                let path = Utf8Path::from_path(entry.path())?;

                // ignore non markdown files, extract name
                let title: String = match path.file_name()?.strip_suffix(".md") {
                    Some(name) => name.to_case(Case::Title),
                    None => return None,
                };

                if let Ok(mut file) = File::open(path) {
                    let mut buf = String::new();
                    if file.read_to_string(&mut buf).is_err() {
                        return None;
                    }
                    let result = matter.parse(&buf);

                    let data = match result.data {
                        Some(p) => match p.as_hashmap() {
                            Ok(h) => h,
                            Err(_) => return None,
                        },
                        None => return None,
                    };

                    Some(Recipe::new(title, data, result.content))
                } else {
                    None
                }
            })
            // exclusion filters
            .filter(|recipe| {
                let pods = &recipe.pods;

                for (exclude_key, maybe_exclude_value) in exclusion_filters.iter() {
                    if let Some(v) = pods.get(exclude_key) {
                        // key found, parse it!
                        let val = match v.as_string() {
                            // key found and parses as a string
                            Ok(val) => val,
                            // key didn't parse, so no need to exclude
                            Err(_) => continue,
                        };

                        // bail if we're looking for a specific value and we find it
                        if let Some(exclude_value) = maybe_exclude_value {
                            if val.eq_ignore_ascii_case(exclude_value) {
                                return false;
                            }
                        } else {
                            // bail if we're not looking for a specific value, because the key matched
                            return false;
                        };
                    }
                }

                true
            })
            // inclusion filters
            .filter(|recipe| {
                let pods = &recipe.pods;

                for (search_key, maybe_search_value) in inclusion_filters.iter() {
                    match pods.get(search_key) {
                        // key found, but we gotta parse it
                        Some(v) => {
                            let val = match v.as_string() {
                                // key found and parses as a string
                                Ok(val) => val,
                                // key didn't parse, so not a match
                                Err(_) => return false,
                            };

                            // bail only if we're looking for a specific value and don't find it
                            if let Some(search_value) = maybe_search_value {
                                if !val.eq_ignore_ascii_case(search_value) {
                                    return false;
                                }
                            };
                        }
                        // key missing from recipe
                        None => return false,
                    };
                }

                true
            });

        let selected_pods: Vec<Recipe> = if let Some(how_many) = self.count {
            let mut rng = thread_rng();

            let pods = pods.choose_multiple(&mut rng, how_many);
            let count_actual = pods.len();

            if count_actual != how_many {
                println!(
                    "Found only {} matching recipes, which is less than {}",
                    count_actual, how_many
                );
            }

            pods
        } else {
            pods.collect()
        };

        for recipe in selected_pods {
            let mut display = recipe.display();

            if self.color.supports_color_on(Stream::Stdout) {
                display.colorize();
            }

            println!("{}", display);
        }

        Ok(())
    }
}
