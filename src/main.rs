mod metrics;
mod persistence;
mod terminal;
use clap::Parser;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};
#[derive(Parser)]
#[command(version, about = "Matrix rain (? toggles live controls)")]
struct Args {
    /// Preferences file (default: platform config directory / termatrix/profiles/default.toml).
    #[arg(long, env = "TERMATRIX_CONFIG")]
    config: Option<PathBuf>,
    /// Weighted color codes, e.g. gG (green) or roygcbm (rainbow).
    #[arg(short = 'c', long, env = "TERMATRIX_COLORS")]
    colors: Option<String>,
    /// Alphabet, e.g. special (Katakana), binary, alpha, or hex.
    #[arg(short = 'g', long, env = "TERMATRIX_GLYPH_SET")]
    glyph_set: Option<String>,
    /// Maximum display refresh rate in Hz (1–240); independent of trace speed.
    #[arg(
        short = 'f',
        long,
        alias = "update-frequency",
        env = "TERMATRIX_MAX_REFRESH_RATE"
    )]
    max_refresh_rate: Option<u16>,
    /// Random seed for reproducible animation with identical simulation inputs.
    #[arg(long, default_value_t = 42)]
    seed: u64,
}
fn main() -> io::Result<()> {
    let a = Args::parse();
    let (path, mut s) = match a.config {
        Some(path) => {
            let settings = persistence::load(&path)?;
            (path, settings)
        }
        None => persistence::load_default()?,
    };
    if let Some(v) = a.colors {
        s.colors = v;
    }
    if let Some(v) = a.glyph_set {
        s.glyph_set = v;
    }
    if let Some(v) = a.max_refresh_rate {
        s.max_refresh_rate = v;
    }
    s.validate().map_err(io::Error::other)?;
    if !io::stdin().is_terminal()
        || !io::stdout().is_terminal()
        || std::env::var("TERM").is_ok_and(|v| v == "dumb")
    {
        return Err(io::Error::other(
            "requires an interactive cursor-capable terminal",
        ));
    }
    terminal::run(s, path, a.seed)
}
