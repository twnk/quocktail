use gray_matter::Pod;
use owo_colors::OwoColorize;
use std::{collections::HashMap, fmt};

use crate::color::Styles;

#[derive(Debug)]
pub struct Recipe {
    /// by convention, Title Case name
    name: String,
    pub pods: HashMap<String, Pod>,
    body: String,
}

impl Recipe {
    pub fn new(name: String, pods: HashMap<String, Pod>, body: String) -> Self {
        Self { name, pods, body }
    }

    /// Returns a type that can display `MyValue`.
    pub fn display(&self) -> RecipeDisplay<'_> {
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
            self.recipe.name.style(self.styles.title_style),
        )?;

        for (k, p) in self.recipe.pods.iter() {
            if let Ok(v) = p.as_string() {
                writeln!(
                    f,
                    "{}: {}",
                    k.style(self.styles.key_style),
                    v.style(self.styles.value_style)
                )?;
            } else {
                // if p.is_empty() {
                writeln!(f, "No {}", k.style(self.styles.key_style))?;
            } // else {
              //     write!(
              //         f,
              //         "Error displaying value of {}. Couldn't figure out {:?}\n",
              //         k.style(self.styles.key_style),
              //         p
              //     )?;
              // }
        }

        write!(f, "{}\n\n", self.recipe.body.style(self.styles.body_style),)
    }
}
