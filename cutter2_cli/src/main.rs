use anyhow::Result;
use clap::Parser;
use cutter2_core::config::template_resolver::FileResolver;
use cutter2_core::config::Config;
use cutter2_core::modes::CutterModeConfig;
use std::fs;
use std::fs::{metadata, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Print paths and operations
    #[clap(short, long, value_parser)]
    verbose: bool,
    /// Output as flat files instead of mirroring directory tree
    #[clap(short, long, value_parser)]
    flatten: bool,
    /// Print debug information
    #[clap(short, long, value_parser)]
    debug: bool,
    /// Output directory of folders
    #[clap(short, long, value_parser, default_value_t = String::from("output"))]
    output: String,
    /// Input directory/file
    #[clap(value_parser)]
    input: String,
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    let Args {
        verbose,
        flatten,
        debug,
        output,
        input,
    } = args;

    let files_to_process: Vec<PathBuf> = if metadata(&input)?.is_file() {
        vec![Path::new(&input).to_path_buf()]
    } else {
        WalkDir::new(&input)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                if let Some(extension) = e.path().extension() {
                    extension == "yaml" || extension == "yml"
                } else {
                    false
                }
            })
            .map(|e| e.into_path())
            .collect()
    };

    for path in files_to_process {
        debug!(path = ?path, "Found yaml at path");
        let in_file_yaml = File::open(path.as_path())?;
        let mut in_yaml_reader = BufReader::new(in_file_yaml);
        let config = Config::load(
            &mut in_yaml_reader,
            FileResolver::new(Path::new("templates"))?,
        )?;
        let mut in_img_path = path.clone();
        in_img_path.set_extension("png");
        let in_img_file = File::open(in_img_path.as_path())?;
        let mut in_img_reader = BufReader::new(in_img_file);

        let out = config.mode.perform_operation(&mut in_img_reader)?;

        for (name_hint, icon) in out {
            let output_path = Path::new(name_hint.as_str());
            let mut file = File::create(output_path)?;
            icon.save(&mut file)?;
        }
    }

    let output_path = Path::new(&output);
    if !output_path.is_dir() {
        fs::create_dir_all(output_path)?;
    }

    Ok(())
}
