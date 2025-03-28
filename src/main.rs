use clap::{Arg, Command};
use glob::Pattern;
use mime_guess::MimeGuess;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

#[derive(Clone, Debug)]
struct TreeItem {
    name: String,
    children: Vec<TreeItem>,
}

impl TreeItem {
    fn new(name: &str) -> Self {
        TreeItem {
            name: name.to_string(),
            children: vec![],
        }
    }

    fn add_child(&mut self, child: TreeItem) {
        self.children.push(child);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("file-lister")
        .version("0.1.0")
        .author("Moonstripe <moonstripe@protonmail.com>")
        .about("Creates LLM friendly text from plaintext files in a directory with an optional prompt.")
        .arg(
            Arg::new("directory")
                .short('d')
                .long("directory")
                .help("Directory to process")
                .required(true),
        )
        .arg(
            Arg::new("prompt")
                .short('p')
                .long("prompt")
                .help("Enable prompts for each file"),
        )
        .arg(
            Arg::new("exclude")
                .short('e')
                .long("exclude")
                .help("Comma-separated list of directories/patterns to exclude (supports glob patterns)")
                .value_parser(clap::value_parser!(String)),
        )
        .get_matches();

    let directory = matches
        .get_one::<String>("directory")
        .expect("Directory is required");

    // Parse exclude patterns
    let exclude_patterns: Vec<Pattern> = matches
        .get_one::<String>("exclude")
        .map(|e| {
            e.split(',')
                .filter_map(|pattern| {
                    Pattern::new(pattern.trim())
                        .map_err(|err| {
                            eprintln!("Warning: Invalid glob pattern '{}': {}", pattern, err);
                            err
                        })
                        .ok()
                })
                .collect()
        })
        .unwrap_or_default();
    println!("### File Tree:");
    // Print the directory tree
    print_tree(directory.as_str(), &exclude_patterns)?;
    println!("\n\n### Files:");
    // Process the files
    list_dir_recursive(Path::new(directory), &exclude_patterns)?;

    if let Some(prompt) = matches.get_one::<String>("prompt") {
        println!("\n\n### Prompt:");
        println!("{}", prompt);
    }

    Ok(())
}

fn is_plain_text_file(path: &Path) -> bool {
    // List of file extensions we want to explicitly consider as plain text
    const PLAIN_TEXT_EXTENSIONS: &[&str] = &[
        // web development
        "ts", "tsx", "js", "jsx", "json", "html", "htm", "css", "scss", "sass",
        // template files
        "twig", "ejs", "hbs", "vue", "svelte", // config files
        "yml", "yaml", "toml", "ini", "env", // documentation
        "md", "markdown", "txt", "rst", // other programming languages
        "py", "rb", "php", "java", "go", "rs", "c", "cpp", "h", "hpp", "sh", "bash",
    ];

    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            // Check against our explicit list first
            if PLAIN_TEXT_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()) {
                return true;
            }
        }
    }

    // Fall back to mime_guess for other files
    match MimeGuess::from_path(path).first() {
        Some(mime_type) => {
            mime_type.type_() == "text"
                || (mime_type.type_() == "application" && (mime_type.subtype() == "json"))
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

fn should_exclude(path: &Path, exclude_patterns: &[Pattern]) -> bool {
    let path_str = path.to_string_lossy();
    exclude_patterns
        .iter()
        .any(|pattern| pattern.matches(&path_str.replace("./", "")))
}

fn build_tree(
    path: &Path,
    exclude_patterns: &[Pattern],
) -> Result<TreeItem, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(path)?;
    let mut root = TreeItem::new(
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_str()
            .unwrap(),
    );

    if metadata.is_dir() && !should_exclude(path, exclude_patterns) {
        let entries = fs::read_dir(path)?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        for entry in entries {
            let child_path = entry.path();
            if !should_exclude(&child_path, exclude_patterns) {
                let child_tree = build_tree(&child_path, exclude_patterns)?;
                root.add_child(child_tree);
            }
        }
    }

    Ok(root)
}

fn print_tree_item(item: &TreeItem, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    println!("{}{}{}", prefix, connector, item.name);

    let new_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    for (i, child) in item.children.iter().enumerate() {
        let is_last_child = i == item.children.len() - 1;
        print_tree_item(child, &new_prefix, is_last_child);
    }
}

fn print_tree(path: &str, exclude_patterns: &[Pattern]) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(path);
    let tree = build_tree(path, exclude_patterns)?;

    println!("{}", path.display());
    for (i, child) in tree.children.iter().enumerate() {
        let is_last_child = i == tree.children.len() - 1;
        print_tree_item(child, "", is_last_child);
    }

    Ok(())
}

fn list_dir_recursive(path: &Path, exclude_patterns: &[Pattern]) -> io::Result<()> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let entry_path = entry.path();
                    if !should_exclude(&entry_path, exclude_patterns) {
                        if entry_path.is_dir() {
                            list_dir_recursive(&entry_path, exclude_patterns)?;
                        } else if is_plain_text_file(&entry_path) {
                            if let Some(_entry_str) = entry_path.to_str() {
                                match read_file(&entry_path) {
                                    Ok(content) => {
                                        let lang = match entry_path
                                            .extension()
                                            .and_then(|ext| ext.to_str())
                                        {
                                            // Web development
                                            Some("ts") | Some("tsx") => "typescript",
                                            Some("js") | Some("jsx") => "javascript",
                                            Some("json") => "json",
                                            Some("html") | Some("htm") => "html",
                                            Some("css") => "css",
                                            Some("scss") | Some("sass") => "scss",

                                            // Template files
                                            Some("twig") => "twig",
                                            Some("ejs") => "ejs",
                                            Some("hbs") => "handlebars",
                                            Some("vue") => "vue",
                                            Some("svelte") => "svelte",

                                            // Config files
                                            Some("yml") | Some("yaml") => "yaml",
                                            Some("toml") => "toml",
                                            Some("ini") => "ini",
                                            Some("env") => "dotenv",

                                            // Documentation
                                            Some("md") | Some("markdown") => "markdown",
                                            Some("txt") => "",
                                            Some("rst") => "restructuredtext",

                                            // Other programming languages
                                            Some("py") => "python",
                                            Some("rb") => "ruby",
                                            Some("php") => "php",
                                            Some("java") => "java",
                                            Some("go") => "go",
                                            Some("rs") => "rust",
                                            Some("c") => "c",
                                            Some("cpp") => "cpp",
                                            Some("h") | Some("hpp") => "cpp",
                                            Some("sh") | Some("bash") => "bash",

                                            _ => "",
                                        };

                                        println!("- {:?}:", entry_path);
                                        println!("```{}", lang);
                                        println!("{}", content);
                                        println!("```");
                                        println!();
                                    }
                                    Err(e) => {
                                        println!("Error reading file {:?}: {}", entry_path, e)
                                    }
                                }
                            }
                        } else {
                            eprintln!("ERROR: {:#?} is not plaintext...", entry_path)
                        }
                    }
                }
                Err(e) => println!("Error reading entry: {}", e),
            }
        }
    }
    Ok(())
}
