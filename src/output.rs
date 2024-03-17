use owo_colors::OwoColorize;
use yaml_rust::Yaml;
use std::fmt;

use crate::color::Styles;
use crate::parse::{Recipe, YAML_OR_KEY};

impl Recipe {
    /// Returns a type that can display `MyValue`.
    pub fn display(&self) -> RecipeDisplay {
        RecipeDisplay {
            recipe: self,
            styles: Box::default(),
        }
    }
}

/// Displayer for [`Recipe`].
pub struct RecipeDisplay<'a> {
    recipe: &'a Recipe,
    styles: Box<Styles>,
}

impl<'a> RecipeDisplay<'a> {
    /// Colorizes the output.
    pub fn colorize(&mut self) {
        self.styles.colorize();
    }
}

impl<'a> fmt::Display for RecipeDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Recipe {}\n\n",
            self.recipe.name().style(self.styles.title_style),
        )?;

        for (k, v) in self.recipe.ingredients().hash.iter() {
            let k = k.as_str().unwrap_or("");
            match v {
                Yaml::Real(_) => todo!(""),
                Yaml::Integer(_) => todo!(),
                Yaml::String(v) => writeln!(
                    f,
                    "{}: {}",
                    k.style(self.styles.key_style),
                    v.style(self.styles.value_style)
                )?,
                Yaml::Boolean(_) => todo!(),
                Yaml::Array(arr) => {
                    writeln!(
                        f,
                        "{}:",
                        k.style(self.styles.key_style),
                    )?;
                    for v in arr {
                        writeln!(
                            f,
                            "  - {}",
                            v.as_str().unwrap_or_default().style(self.styles.value_style),
                        )?;
                    }
                },
                Yaml::Hash(hash) => {
                    if let Some(Yaml::Array(or_arr)) = hash.get(&YAML_OR_KEY) {
                        writeln!(
                            f,
                            "{} (choice of):",
                            k.style(self.styles.key_style),
                        )?;
                        for v in or_arr {
                            writeln!(
                                f,
                                "  - {}",
                                v.as_str().unwrap_or_default().style(self.styles.value_style),
                            )?;
                        }
                    } else {
                        writeln!(
                            f,
                            "unexpected format {}: {:?}",
                            k.style(self.styles.key_style),
                            v
                        )?;
                    }
                },
                Yaml::Alias(_) => todo!(),
                Yaml::Null => {},
                Yaml::BadValue => todo!(),
            };
        }

        write!(
            f,
            "{}\n\n",
            self.recipe.instructions().style(self.styles.body_style),
        )
    }
}
