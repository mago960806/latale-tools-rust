use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use latale_tools::spf::{SpfReader, SpfRegistry, SpfWriter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "latale-spf")]
#[command(about = "LaTale SPF resource pack/unpack tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Unpack SPF file to directory
    Unpack {
        /// SPF file to unpack
        spf_file: PathBuf,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Pack directory into SPF file
    Pack {
        /// SPF file name to create (determines FILE_ID and source path)
        spf_file: String,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Source DATA directory (default: ./DATA)
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Unpack { spf_file, output } => {
            cmd_unpack(&spf_file, output.as_deref())?;
        }
        Commands::Pack { spf_file, output, data_dir } => {
            cmd_pack(&spf_file, output.as_deref(), data_dir.as_deref())?;
        }
    }

    Ok(())
}

fn cmd_unpack(spf_file: &std::path::Path, output: Option<&std::path::Path>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| std::path::Path::new("."));

    println!("Opening {}...", spf_file.display());
    let reader = SpfReader::open(spf_file)
        .with_context(|| format!("Failed to open SPF: {}", spf_file.display()))?;

    println!("SPF version: {}", reader.version());
    println!("File ID: {}", reader.header().file_id);
    println!("Description: {}", reader.header().desc_str());
    println!("File count: {}", reader.file_count());

    println!("Unpacking to {}...", output_dir.display());
    reader.unpack(output_dir)
        .context("Failed to unpack SPF")?;

    println!("Done!");
    Ok(())
}

fn cmd_pack(spf_name: &str, output: Option<&std::path::Path>, data_dir: Option<&std::path::Path>) -> Result<()> {
    let registry = SpfRegistry::find_by_name(spf_name)
        .with_context(|| format!("Unknown SPF name: {}", spf_name))?;

    let data_dir = data_dir.unwrap_or_else(|| std::path::Path::new("DATA"));
    let output_dir = output.unwrap_or_else(|| std::path::Path::new("."));

    println!("Packing {} (FILE_ID={})", registry.name, registry.file_id);
    println!("Source: {}/{}", data_dir.display(), registry.path_prefix);

    let mut writer = SpfWriter::new(registry.file_id);
    writer.add_from_dir(data_dir, registry.path_prefix)
        .context("Failed to read source files")?;

    let output_path = output_dir.join(format!("{}.SPF", registry.name));
    println!("Output: {}", output_path.display());

    writer.write(&output_path)
        .context("Failed to write SPF")?;

    println!("Done!");
    Ok(())
}
