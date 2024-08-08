//! Executable entry
use anyhow::Result;
use clap::{Parser, Subcommand};
use jcat::compress::CType;
use jcat::{run_cat, run_data_definition};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Concatenates files in directories
    Cat {
        /// directory to read from
        #[arg(short, long)]
        in_dir: PathBuf,

        /// directory to write to
        #[arg(short, long)]
        out_dir: PathBuf,

        /// perform action on all subdirectories
        #[arg(short, long)]
        recursive: bool,

        ///// Compression to apply on write
        #[arg(short, long, default_value_t, value_enum)]
        write_compression: CTypeCmd,
    },
    /// Provides type for deserializing output
    DataDefinition {},
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum CTypeCmd {
    /// No compression
    #[default]
    Raw,
    /// Gzip compression
    Gzip,
    /// Snappy compression
    Snappy,
}

impl From<CTypeCmd> for CType {
    fn from(ctc: CTypeCmd) -> Self {
        match ctc {
            CTypeCmd::Raw => Self::Raw,
            CTypeCmd::Gzip => Self::Gzip,
            CTypeCmd::Snappy => Self::Snappy,
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Cat {
            in_dir,
            out_dir,
            recursive,
            write_compression,
        } => run_cat::<serde_json::Value>(in_dir, out_dir, recursive, write_compression.into()),
        Commands::DataDefinition {} => run_data_definition(),
    }
}
