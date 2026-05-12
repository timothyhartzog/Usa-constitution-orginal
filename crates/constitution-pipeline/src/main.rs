#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use constitution_pipeline::{validate, PipelinePaths, Snapshot};

#[derive(Debug, Parser)]
#[command(name = "constitution-pipeline", version, about)]
struct Cli {
    /// Repository root.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Snapshot canonical data files without modifying them.
    Snapshot {
        /// Output snapshot path.
        #[arg(long, default_value = "data/pipeline/snapshot.json")]
        output: PathBuf,
    },
    /// Validate canonical corpus JSON and report counts.
    Validate,
    /// Verify current files match a previously written snapshot.
    VerifySnapshot {
        /// Snapshot path to compare against.
        #[arg(long, default_value = "data/pipeline/snapshot.json")]
        snapshot: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = PipelinePaths::new(cli.root);
    match cli.cmd {
        Cmd::Snapshot { output } => {
            let snapshot = Snapshot::build(&paths)?;
            let output = paths.root.join(output);
            snapshot.write_pretty(&output)?;
            println!(
                "wrote {} records to {}",
                snapshot.records.len(),
                output.display()
            );
        }
        Cmd::Validate => {
            let report = validate::validate_corpus(&paths)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Cmd::VerifySnapshot { snapshot } => {
            let expected = Snapshot::read(&paths.root.join(snapshot))?;
            let actual = Snapshot::build(&paths)?;
            if expected == actual {
                println!("snapshot verified: {} records", actual.records.len());
            } else {
                anyhow::bail!(
                    "snapshot mismatch: expected {} records, found {} records",
                    expected.records.len(),
                    actual.records.len()
                );
            }
        }
    }
    Ok(())
}
