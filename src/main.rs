//! Entry point for `mdfmt`.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use markdown_formatter::{Mode, Outcome, Wrap, format_file};
use tracing_subscriber::EnvFilter;

/// Command-line arguments.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Markdown files to format in place.
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    /// Report files that would change instead of writing them;
    /// exit 1 if any would.
    #[arg(long)]
    check: bool,
    /// Put each paragraph on one line instead of wrapping at 79 columns.
    #[arg(long)]
    unwrap: bool,
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

    let mode = if args.check { Mode::Check } else { Mode::Write };
    let wrap = if args.unwrap {
        Wrap::Unwrap
    } else {
        Wrap::Fill
    };
    let mut failed = false;

    for path in &args.paths {
        match format_file(path, mode, wrap) {
            Ok(Outcome::NeedsFormatting) => {
                println!("{}", path.display());
                failed = true;
            }
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
