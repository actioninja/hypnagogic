mod error;

use std::fs;
use std::fs::{metadata, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::exit;

use anyhow::Result;
use clap::Parser;
use image::DynamicImage;
use rayon::prelude::*;
use tracing::{info, Level};
use user_error::UFE;
use walkdir::WalkDir;

use crate::error::Error;
use hypnagogic_core::config::error::ConfigError;
use hypnagogic_core::config::template_resolver::error::TemplateError;
use hypnagogic_core::config::template_resolver::file_resolver::FileResolver;
use hypnagogic_core::config::Config;
use hypnagogic_core::modes::CutterModeConfig;

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

fn main() -> Result<()> {
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

    let num_files = files_to_process.len();
    println!("Found {} files!", num_files);

    let result = files_to_process
        .par_iter()
        .try_for_each(|path| process_icon(flatten, debug, &output, &templates, path));

    if let Err(err) = result {
        err.into_ufe().print();
        if !dont_wait {
            dont_disappear::any_key_to_continue::default();
            exit(1);
        }
    }

    println!("Successfully processed {} files!", num_files);

    if !dont_wait {
        dont_disappear::any_key_to_continue::default();
    }

    Ok(())
}

/// Gnarly, effectful function hoisted out here so that I can still use ? but parallelize with rayon
fn process_icon(
    flatten: bool,
    debug: bool,
    output: &Option<String>,
    templates: &String,
    path: &PathBuf,
) -> Result<(), Error> {
    info!(path = ?path, "Found yaml at path");
    let in_file_yaml = File::open(path.as_path())?;
    let mut in_yaml_reader = BufReader::new(in_file_yaml);
    let config = Config::load(
        &mut in_yaml_reader,
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
            ConfigError::TemplateError(template_err) => {
                if let TemplateError::FailedToFindTemplate(template_string, expected_path) =
                    template_err
                {
                    Error::TemplateNotFound {
                        source_config,
                        template_string,
                        expected_path,
                    }
                } else {
                    Error::InvalidConfig {
                        source_config,
                        cause: "Some Cause".to_string(),
                    }
                }
            }
            ConfigError::YamlError(_err) => Error::InvalidConfig {
                source_config,
                cause: "Invalid Config".to_string(),
            },
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

    //TODO: Operation error handling
    let out = config.mode.perform_operation(&mut in_img_reader).unwrap();

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

    // Real nasty code here. Lots of unwraps. Debug is only intended to be used in a dev environment,
    // so panics are acceptable
    if debug {
        let in_img_file = File::open(in_img_path.as_path())?;
        let mut debug_reader = BufReader::new(in_img_file);
        let mut debug_output_dir = in_img_path.parent().unwrap().to_path_buf();
        let file_name = in_img_path.file_name().unwrap().to_str().unwrap();
        debug_output_dir.push(Path::new(format!("{file_name}-DEBUG").as_str()));
        fs::create_dir_all(debug_output_dir.as_path())?;
        let debug_out: DynamicImage = config
            .mode
            .debug_output(&mut debug_reader, debug_output_dir)
            .unwrap();
        let mut debug_path = in_img_path.clone();
        debug_path.set_extension("");
        let current_file_name = debug_path.file_name().unwrap().to_str().unwrap();
        debug_path.set_file_name(format!("{current_file_name}-DEBUGOUT"));
        debug_path.set_extension("png");

        process_path(&mut debug_path);

        info!(path = ?debug_path, "Writing debug");
        debug_out.save(debug_path).unwrap();
    }

    let prefix = config.file_prefix.unwrap_or_else(|| "".to_string());
    for (name_hint, icon) in out {
        let name_hint = name_hint
            .map(|nh| format!("-{nh}"))
            .unwrap_or_else(|| "".to_string());
        let mut new_path = in_img_path.clone();
        new_path.set_extension("");
        let current_file_name = new_path.file_name().unwrap().to_str().unwrap();
        let built_filename = format!("{prefix}{current_file_name}{name_hint}.dmi");
        new_path.pop();
        new_path.push(built_filename);
        info!(name_hint = ?name_hint, path = ?new_path, "Writing output");

        process_path(&mut new_path);

        let parent_dir = new_path.parent().expect(
            "Failed to get parent? (this is a program error, not a config error! Please report!)",
        );

        fs::create_dir_all(parent_dir).expect(
            "Failed to create dirs (This is a program error, not a config error! Please report!",
        );

        let mut file = File::create(new_path).expect("Failed to create output file (This is a program error, not a config error! Please report!)");

        //TODO: figure out a better thing to do than just the unwrap
        icon.save(&mut file).unwrap();
    }
    Ok(())
}
