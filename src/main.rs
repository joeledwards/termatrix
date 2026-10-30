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
    #[arg(long, env = "TERMATRIX_CONFIG")]
    config: Option<PathBuf>,
    #[arg(short = 'c', long, env = "TERMATRIX_COLORS")]
    colors: Option<String>,
    #[arg(short = 'g', long, env = "TERMATRIX_GLYPH_SET")]
    glyph_set: Option<String>,
    #[arg(
        short = 'f',
        long,
        alias = "update-frequency",
        env = "TERMATRIX_MAX_REFRESH_RATE"
    )]
    max_refresh_rate: Option<u16>,
    #[arg(long, default_value_t = 42)]
    seed: u64,
}
fn main() -> io::Result<()> {
    let a = Args::parse();
    let path = a.config.map(Ok).unwrap_or_else(persistence::default_path)?;
    let mut s = persistence::load(&path)?;
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
