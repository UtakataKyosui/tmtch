use std::path::Path;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "
Rustで実装された、拡張子毎にテンプレートを設定可能なファイル作成ツール
EN: A file creation tool implemented in Rust that allows setting templates for each file extension.
"
)]
struct Arg {
    #[command(subcommand)]
    subcommand: Option<Commands>,
    /// The name of the file to operate on (required for the Edit command)
    name: Option<String>,
}
#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    templates: std::collections::HashMap<String, String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new file with the specified name
    Touch,
    /// Print the extension of the file
    Edit { name: String },
    /// List all available templates
    List,
}

fn main() {
    let args = Arg::parse();

    if let Some(subcommand) = &args.subcommand {
        match subcommand {
            Commands::Edit { name } => {
                if !name.is_empty() {
                    edit_template(name);
                    return;
                } else {
                    println!("No file name provided for editing.");
                    return;
                }
            }
            Commands::List => {
                if let Ok(config) = load_template() {
                    for (ext, template) in config.templates {
                        let template = template.trim();
                        if template.is_empty() {
                            println!("Extension: {}, Template is empty", ext);
                            return;
                        }
                        println!("Extension: {}, Template: {}", ext, template);
                    }
                } else {
                    println!("Failed to load templates.");
                }
                return;
            }
            _ => {
                if let Some(file) = &args.name {
                    if let Some(help) = get_version(file) {
                        println!("Version for file {}: {}", file, help);
                    } else if let Some(extension) = get_extension(file) {
                        println!("File: {}", file);
                        println!("Extension: {}", extension);
                    } else {
                        println!("No extension found for file: {}", file);
                    }
                } else {
                    println!("No file provided.");
                }
            }
        }
    }
}

fn get_extension(file: &String) -> Option<&str> {
    let path = Path::new(file);
    path.extension().and_then(|ext| ext.to_str())
}

fn get_version(file: &String) -> Option<&str> {
    // Implement your logic to determine if version information is available for the file
    // Return Some(version_text) if version information is available, otherwise None
    None
}

fn edit_template(name: &String) {
    println!("Editing template for file: {}", name);
}

fn load_template() -> Result<Config, confy::ConfyError> {
    let config: Config = confy::load("tmtch", None)?;
    Ok(config)
}
