use anyhow::Result;
use clap::Parser;
use cutter2_core::config::template_resolver::FileResolver;
use cutter2_core::config::Config;
use cutter2_core::modes::CutterModeConfig;
use image::DynamicImage;
use std::fs;
use std::fs::{metadata, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tracing::{info, Level};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Print paths and operations
    #[clap(short, long, value_parser)]
    verbose: bool,
    /// Output as flat files instead of mirroring directory tree
    #[clap(short, long, value_parser)]
    flatten: bool,
    /// Print debug information and produce debug outputs
    #[clap(short, long, value_parser)]
    debug: bool,
    /// Doesn't wait for a keypress after running. For CI or toolchain usage.
    #[clap(short = 'w', long, value_parser)]
    dont_wait: bool,
    /// Output directory of folders
    #[clap(short, long, value_parser)]
    output: Option<String>,
    /// Input directory/file
    #[clap(value_parser)]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let Args {
        verbose,
        flatten,
        debug,
        dont_wait,
        output,
        input,
    } = args;

    if debug {
        let subscriber = tracing_subscriber::fmt()
            .pretty()
            .with_max_level(Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else if verbose {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .compact()
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_max_level(Level::WARN)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    };

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
        info!(path = ?path, "Found yaml at path");
        let in_file_yaml = File::open(path.as_path())?;
        let mut in_yaml_reader = BufReader::new(in_file_yaml);
        let config = Config::load(
            &mut in_yaml_reader,
            FileResolver::new(Path::new("templates"))?,
        )?;
        let mut in_img_path = path.clone();
        in_img_path.set_extension("png");
        let in_img_file = File::open(in_img_path.as_path())?;
        let mut in_img_reader = BufReader::new(in_img_file.try_clone()?);

        let out = config.mode.perform_operation(&mut in_img_reader)?;

        if let Some(output) = &output {
            let output_path = Path::new(output);
            fs::create_dir_all(output_path)?;
        }

        let process_path = |path: &mut PathBuf| {
            if flatten {
                let file_name = path.file_name().map(|s| s.to_os_string()).unwrap();
                path.clear();
                path.push(file_name);
            }

            if let Some(output) = &output {
                let buf = PathBuf::from(output).join(&path);
                path.clear();
                path.push(buf);
            }
        };

        if debug {
            let in_img_file = File::open(in_img_path.as_path())?;
            let mut debug_reader = BufReader::new(in_img_file);
            let debug_out: DynamicImage = config.mode.debug_output(&mut debug_reader)?;
            let mut debug_path = in_img_path.clone();
            debug_path.set_extension("");
            let current_file_name = debug_path.file_name().unwrap().to_str().unwrap();
            debug_path.set_file_name(format!("{current_file_name}-DEBUGOUT"));
            debug_path.set_extension("png");

            process_path(&mut debug_path);

            info!(path = ?debug_path, "Writing debug");
            debug_out.save(debug_path)?
        }

        let prefix = config.file_prefix.unwrap_or_else(|| "".to_string());
        for (name_hint, icon) in out {
            let mut new_path = in_img_path.clone();
            let current_file_name = new_path.file_name().unwrap().to_str().unwrap();
            new_path.set_file_name(format!("{prefix}{current_file_name}{name_hint}"));
            new_path.set_extension("dmi");
            info!(path = ?new_path, "Writing output");

            process_path(&mut new_path);

            let mut file = File::create(new_path)?;

            icon.save(&mut file)?;
        }
    }

    if !dont_wait {
        dont_disappear::any_key_to_continue::default();
    }

    Ok(())
}
