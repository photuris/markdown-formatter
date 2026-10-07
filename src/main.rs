//! Entry point for `mdfmt`.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use markdown_formatter::{Outcome, format_file};
use tracing_subscriber::EnvFilter;

/// Command-line arguments.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Markdown files to re-wrap in place.
    #[arg(required = true)]
    paths: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    let mut failed = false;

    for path in &args.paths {
        match format_file(path) {
            Ok(Outcome::SkippedVault) => tracing::warn!(
                path = %path.display(),
                "skipped: inside an Obsidian vault"
            ),
            Ok(Outcome::Unchanged | Outcome::Rewritten) => {}
            Err(e) => {
                tracing::error!("{e}");
                failed = true;
            }
        }
    }

    if failed {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
