mod error;

use std::fs;
use std::fs::{metadata, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Instant;

use anyhow::{anyhow, Result};
use clap::Parser;
use rayon::prelude::*;
use tracing::{debug, info, Level};
use user_error::UFE;
use walkdir::WalkDir;

use crate::error::Error;
use hypnagogic_core::config::error::ConfigError;
use hypnagogic_core::config::read_config;
use hypnagogic_core::config::template_resolver::error::TemplateError;
use hypnagogic_core::config::template_resolver::file_resolver::FileResolver;
use hypnagogic_core::operations::{
    IconOperationConfig, NamedIcon, OperationMode, OutputImage, ProcessorPayload,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Print paths and operations
    #[arg(short, long)]
    verbose: bool,
    /// Output as flat files instead of mirroring directory tree
    #[arg(short, long)]
    flatten: bool,
    /// Print debug information and produce debug outputs
    #[arg(short, long)]
    debug: bool,
    /// Doesn't wait for a keypress after running. For CI or toolchain usage.
    #[arg(short = 'w', long)]
    dont_wait: bool,
    /// Output directory of folders. If not set, output will match the file tree and output adjacent to input
    #[arg(short, long)]
    output: Option<String>,
    /// Location of the templates folder
    #[arg(short, long, default_value_t = String::from("templates"))]
    templates: String,
    /// Input directory/file
    input: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let now = Instant::now();
    let args = Args::parse();
    let Args {
        verbose,
        flatten,
        debug,
        dont_wait,
        output,
        templates,
        input,
    } = args;

    println!("Hypnagogic CLI v{VERSION}");

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

    if !Path::new(&input).exists() {
        return Err(anyhow!("Input path does not exist!"));
    }

    let files_to_process: Vec<PathBuf> = if metadata(&input)?.is_file() {
        vec![Path::new(&input).to_path_buf()]
    } else {
        WalkDir::new(&input)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                if let Some(extension) = e.path().extension() {
                    extension == "toml"
                } else {
                    false
                }
            })
            .map(|e| e.into_path())
            .collect()
    };
    debug!(files = ?files_to_process, "Files to process");

    let num_files = files_to_process.len();
    println!("Found {num_files} files!");

    let result: Result<Vec<()>, Error> = files_to_process
        .par_iter()
        .map(|path| process_icon(flatten, debug, &output, &templates, path))
        .collect();

    if let Err(err) = result {
        err.into_ufe().print();
        if !dont_wait {
            dont_disappear::any_key_to_continue::default();
            exit(1);
        }
    }

    println!(
        "Successfully processed {num_files} files! (Took {:.2?})",
        now.elapsed()
    );

    if !dont_wait {
        dont_disappear::any_key_to_continue::default();
    }

    Ok(())
}

/// Gnarly, effectful function hoisted out here so that I can still use ? but parallelize with rayon
#[allow(clippy::result_large_err)]
fn process_icon(
    flatten: bool,
    debug: bool,
    output: &Option<String>,
    templates: &String,
    path: &PathBuf,
) -> Result<(), Error> {
    info!(path = ?path, "Found toml at path");
    let in_file_toml = File::open(path.as_path())?;
    let mut in_toml_reader = BufReader::new(in_file_toml);
    let config = read_config(
        &mut in_toml_reader,
        FileResolver::new(Path::new(&templates))
            .map_err(|_err| Error::NoTemplateFolder(PathBuf::from(templates)))?,
    )
    .map_err(|err| {
        let source_config = path
            .clone()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        match err {
            ConfigError::Template(template_err) => match template_err {
                TemplateError::FailedToFindTemplate(template_string, expected_path) => {
                    Error::TemplateNotFound {
                        source_config,
                        template_string,
                        expected_path,
                    }
                }
                TemplateError::TOMLError(err) => Error::InvalidConfig {
                    source_config,
                    config_error: err.into(),
                },
                TemplateError::IOError(err) => err.into(),
            },
            ConfigError::Toml(err) => Error::InvalidConfig {
                source_config,
                config_error: ConfigError::Toml(err),
            },
            ConfigError::Config(_) => Error::InvalidConfig {
                source_config,
                config_error: err,
            },
            _ => panic!("Unexpected error: {:#?}", err),
        }
    })?;
    let mut in_img_path = path.clone();
    in_img_path.set_extension("png");
    let in_img_file = File::open(in_img_path.as_path()).map_err(|_err| {
        let source_config = path
            .clone()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let expected = in_img_path
            .clone()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let search_dir = path.clone().parent().unwrap().to_path_buf();
        Error::InputNotFound {
            source_config,
            expected,
            search_dir,
        }
    })?;
    let mut in_img_reader = BufReader::new(in_img_file.try_clone()?);

    let mode = if debug {
        OperationMode::Debug
    } else {
        OperationMode::Standard
    };

    //TODO: Operation error handling
    let out: ProcessorPayload = config.do_operation(&mut in_img_reader, mode).unwrap();

    if let Some(output) = &output {
        let output_path = Path::new(output);
        fs::create_dir_all(output_path)?;
    }

    let process_path = |path: PathBuf, named_img: Option<&NamedIcon>| -> PathBuf {
        debug!(path = ?path, img = ?named_img, "Processing path");
        let processed_path = if let Some(named_img) = named_img {
            named_img.build_path(path.as_path())
        } else {
            PathBuf::from(path.file_name().unwrap().to_str().unwrap().to_string())
        };
        debug!(path = ?processed_path, "Processed path");

        let parent_path = path.parent().unwrap();

        let mut path = PathBuf::new();

        if let Some(output) = &output {
            path = PathBuf::from(output).join(&path);
        }

        if !flatten {
            path.push(parent_path);
        }
        path.push(processed_path);
        info!(path = ?path, "Processed path");

        path
    };

    let mut out_paths: Vec<(PathBuf, OutputImage)> = vec![];

    match out {
        ProcessorPayload::Single(inner) => {
            let mut processed_path = process_path(in_img_path.clone(), None);
            processed_path.set_extension(inner.extension());
            out_paths.push((processed_path, *inner));
        }
        ProcessorPayload::SingleNamed(named) => {
            let mut processed_path = process_path(in_img_path.clone(), Some(&named));
            processed_path.set_extension(named.image.extension());
            out_paths.push((processed_path, named.image))
        }
        ProcessorPayload::MultipleNamed(icons) => {
            for icon in icons {
                let mut processed_path = process_path(in_img_path.clone(), Some(&icon));
                processed_path.set_extension(icon.image.extension());
                out_paths.push((processed_path, icon.image))
            }
        }
    }

    for (mut path, icon) in out_paths {
        let parent_dir = path.parent().expect(
            "Failed to get parent? (this is a program error, not a config error! Please report!)",
        );

        fs::create_dir_all(parent_dir).expect(
            "Failed to create dirs (This is a program error, not a config error! Please report!)",
        );

        let mut file = File::create(path.as_path()).expect("Failed to create output file (This is a program error, not a config error! Please report!)");

        //TODO: figure out a better thing to do than just the unwrap
        match icon {
            OutputImage::Png(png) => {
                png.save(&mut path).unwrap();
            }
            OutputImage::Dmi(dmi) => {
                dmi.save(&mut file).unwrap();
            }
        }
    }
    Ok(())
}
