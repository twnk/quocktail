use owo_colors::Style;

// Stylesheet used to colorize MyValueDisplay below.
#[derive(Debug, Default)]
pub struct Styles {
    pub title_style: Style,
    pub key_style: Style,
    pub value_style: Style,
    pub body_style: Style,
    pub err_style: Style,
    // ... other styles
}

impl Styles {
    pub fn colorize(&mut self) {
        self.title_style = Style::new().bold().bright_purple();
        self.key_style = Style::new().bright_purple();
        self.value_style = Style::new().italic().bright_green();
        self.body_style = Style::new().bright_green();
        self.err_style = Style::new().bright_red();
        // ... other styles
    }
}
