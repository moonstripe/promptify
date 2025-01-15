use clap::{Arg, Command};
use mime_guess::MimeGuess;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Lists files in a directory recursively with optional prompts
fn main() -> io::Result<()> {
    // Set up clap with the correct syntax
    let matches = Command::new("file-lister")
        .version("0.1.0")
        .author("Moonstripe <moonstripe@protonmail.com>")
        .about("Creates LLM friendly text from plaintext files in a directory with an optional prompt.")
        .arg(
            Arg::new("directory")
                .short('d')
                .long("directory")
                .help("Directory to process"),
        )
        .arg(
            Arg::new("prompt")
                .short('p')
                .long("prompt")
                .help("Enable prompts for each file"),
        )
        .get_matches();

    // Run directory formatting
    if let Some(directory) = matches.get_one::<String>("directory") {
        // Start the recursive listing from the provided directory

        list_dir_recursive(Path::new(directory))?;
    };

    if let Some(prompt) = matches.get_one::<String>("prompt") {
        println!("{}", prompt)
    };

    Ok(())
}

fn is_plain_text_file(path: &Path) -> bool {
    // First, check if the file extension is .html.twig (considered plain text)
    if let Some(ext) = path.extension() {
        if ext == "twig" {
            return true; // Treat .html.twig files as plain text
        }
    }

    // Use mime_guess to determine the MIME type of the file
    match MimeGuess::from_path(path).first() {
        Some(mime_type) => {
            mime_type.type_() == "text"
                || (mime_type.type_() == "application" && mime_type.subtype() == "json")
        }
        None => false,
    }
}

fn read_file(path: &Path) -> io::Result<String> {
    let mut content = String::new();
    let mut file = fs::File::open(path)?;
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn list_dir_recursive(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let entry_path = entry.path();
                    // If it's a directory, recurse; if it's a valid file, read it
                    if entry_path.is_dir() {
                        list_dir_recursive(&entry_path)?;
                    } else if is_plain_text_file(&entry_path) {
                        if let Some(entry_str) = entry_path.to_str() {
                            match read_file(&entry_path) {
                                Ok(content) => {
                                    println!("```{}", entry_str);
                                    println!("{}", content);
                                    println!("```");
                                    println!();
                                }
                                Err(e) => println!("Error reading file {:?}: {}", entry_path, e),
                            }
                        }
                    } else {
                        eprintln!("ERROR: {:#?} is not plaintext...", entry_path)
                    }
                }
                Err(e) => println!("Error reading entry: {}", e),
            }
        }
    }
    Ok(())
}
