mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Encode {
        file_path: PathBuf,
        chunk_type: String,
        message: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    Decode {
        file_path: PathBuf,
        chunk_type: String,
    },
    Remove {
        file_path: PathBuf,
        chunk_type: String,
    },
    Print {
        file_path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Encode {
            file_path,
            chunk_type,
            message,
            output,
        } => {
            println!("Encoding message into {}", file_path.display());
            println!("Chunk type: {}", chunk_type);
            println!("Message: {}", message);
            if let Some(out) = output {
                println!("Output file: {}", out);
            }
        }
        Commands::Decode {
            file_path,
            chunk_type,
        } => {
            println!("Decoding message from {}", file_path.display());
            println!("Chunk type: {}", chunk_type);
        }
        Commands::Remove {
            file_path,
            chunk_type,
        } => {
            println!("Removing chunk from {}", file_path.display());
            println!("Chunk type: {}", chunk_type);
        }
        Commands::Print { file_path } => {
            println!("Printing chunks from {}", file_path.display());
        }
    }
}
