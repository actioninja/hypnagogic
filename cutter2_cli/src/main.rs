use std::fs;
use std::fs::metadata;
use std::io::{self};
use std::path::{Path, PathBuf};
use clap::Parser;
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

fn main() -> io::Result<()> {
    let args = Args::parse();
    let Args { verbose, flatten, debug, output, input } = args;

    let files_to_process: Vec<PathBuf> = if metadata(&input)?.is_file() {
        vec![Path::new(&input).to_path_buf()]
    } else {
        WalkDir::new(&input).into_iter()
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
        println!("Pathbuf: {:?}", path);
    }

    let output_path = Path::new(&output);
    if !output_path.is_dir() {
        fs::create_dir_all(output_path)?;
    }

    Ok(())
}
