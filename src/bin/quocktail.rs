use clap::Parser;
use quocktail::Quocktail;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // 🤞 that supports_color can detect if this succeeds or fails.
    // TODO: Can't test windows rn!
    let _ = enable_ansi_support::enable_ansi_support();

    let qu = Quocktail::parse();

    qu.exec()
}
