mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Pingu {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Encode {
        #[arg(short, long)]
        png: PathBuf,
        #[arg(short, long)]
        message: String,
        #[arg(short, long)]
        chunk_type: chunk_type::ChunkType,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Decode {
        #[arg(short, long)]
        png: PathBuf,
        #[arg(short, long)]
        chunk_type: chunk_type::ChunkType,
    },
    Remove {
        #[arg(short, long)]
        png: PathBuf,
        #[arg(short, long)]
        chunk_type: chunk_type::ChunkType,
    },
    Print {
        #[arg(short, long)]
        png: PathBuf,
    },
}

#[allow(unused_variables, dead_code)]
fn main() -> Result<()> {
    let cli = Pingu::parse();

    match cli.command {
        Some(Commands::Encode {
            png,
            message,
            chunk_type,
            output,
        }) => {
            //read the png file into byte slice
            let png_data = std::fs::read(&png)?;

            let chunk = chunk::Chunk::new(chunk_type, message.as_bytes().to_vec());
            let mut png = png::Png::try_from(png_data.as_slice())?;
            png.append_chunk(chunk);

            let png_bytes = png.as_bytes();
            if let Some(output) = output {
                std::fs::write(&output, &png_bytes)?;
            } else {
                println!("{}", png);
            }
            Ok(())
        }
        Some(Commands::Decode { png, chunk_type }) => {
            let png_data = std::fs::read(&png)?;
            let png = png::Png::try_from(png_data.as_slice())?;

            let chunk = png.chunk_by_type(chunk_type.to_string().as_str());
            if let Some(chunk) = chunk {
                let message = chunk.data_as_string()?;
                println!("{}", message);
            } else {
                println!("Chunk not found");
            }

            Ok(())
        }
        Some(Commands::Remove { png, chunk_type }) => {
            let png_data = std::fs::read(&png)?;
            let mut png = png::Png::try_from(png_data.as_slice())?;

            let removed_chunk = png.remove_chunk(chunk_type.to_string().as_str())?;

            println!("{}", removed_chunk);

            Ok(())
        }
        Some(Commands::Print { png }) => {
            let png_data = std::fs::read(&png)?;
            let png = png::Png::try_from(png_data.as_slice())?;

            println!("{}", png);

            Ok(())
        }
        None => {
            println!("No command provided");
            Ok(())
        }
    }
}
