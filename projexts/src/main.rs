use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use serde::{Serialize, Deserialize};
use serde_json;


fn config_file_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".projexts_config.json")
}

fn load_shortcuts() -> io::Result<Vec<Shortcut>> {
    let path = config_file_path();
    if !path.exists() {
        println!("Creating storage for shortcuts...");
        // Create an empty file if it doesn't exist
        fs::File::create(&path)?.write_all(b"[]")?;
    }
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

fn save_shortcuts(shortcuts: &[Shortcut]) -> io::Result<()>{
    let data = serde_json::to_string_pretty(shortcuts)?;
    fs::write(config_file_path(), data)?;
    Ok(())
}

fn add_shortcut(name: &str, command: &str) -> io::Result<()> {
    let mut shortcuts = load_shortcuts()?;
    shortcuts.push(Shortcut {
        project_name: name.to_string(),
        run_command: command.to_string(),
    });
    save_shortcuts(&shortcuts)
}

fn remove_shortcut(name: &str) -> io::Result<()> {
    let mut shortcuts = load_shortcuts()?;
    let initial_len = shortcuts.len();
    
    // Retain only shortcuts that do not match the given name
    shortcuts.retain(|shortcut| shortcut.project_name != name);
    
    if shortcuts.len() == initial_len {
        println!("No shortcut found with name '{}'.", name);
    } else {
        println!("Shortcut '{}' removed successfully.", name);
        save_shortcuts(&shortcuts)?;
    }
    Ok(())
}

fn list_shortcuts() -> io::Result<()> {
    let shortcuts = load_shortcuts()?;

    if shortcuts.len() == 0 {
        println!("No shortcuts found");
    }

    for shortcut in shortcuts {
        println!("{}: {}", shortcut.project_name, shortcut.run_command);
    }
    Ok(())
}

fn run_shortcut(name: &str) -> io::Result<()> {
    let shortcuts = load_shortcuts()?;
    if let Some(shortcut) = shortcuts.iter().find(|s| s.project_name == name) {
        println!("Running command: {}", shortcut.run_command);

        // Split the command into parts for Command execution
        let mut parts = shortcut.run_command.split_whitespace();
        if let Some(command) = parts.next() {
            let args: Vec<&str> = parts.collect();
            Command::new(command)
                .args(&args)
                .spawn()?
                .wait()?;  // Wait for the command to complete
        } else {
            eprintln!("Error: Command for '{}' is empty.", name);
        }
    } else {
        eprintln!("Error: No shortcut found with name '{}'", name);
    }
    Ok(())
}


#[derive(Serialize, Deserialize, Clone)]
struct Shortcut {
    project_name: String,
    run_command: String,
}

#[derive(Parser)]
#[command(name = "projexts", about = "A CLI tool to manage project shortcuts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new shortcut
    Add {
        /// Name of the project
        name: String,
        /// Command to run the project
        command: String,
    },
    /// Removes a shortcut
    Remove {
        /// Name of the project
        name: String,
    },
    /// List all shortcuts
    List,
    /// Run a shortcut by name
    Run {
        /// Name of the project to run
        name: String,
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Add { name, command } => {
            println!("Adding shortcut: {} -> {}", name, command);
            if let Err(e) = add_shortcut(&name, &command) {
                eprintln!("Failed to add shortcut: {}", e);
            }
        }
        Commands::Remove { name } => {
            println!("Removing shortcut: {}", name);
            if let Err(e) = remove_shortcut(&name) {
                eprintln!("Failed to remove shortcut: {}", e);
            }
        }
        Commands::List => {
            if let Err(e) = list_shortcuts() {
                eprintln!("Failed to list shortcuts: {}", e);
            }
        }
        Commands::Run { name } => {
            if let Err(e) = run_shortcut(&name) {
                eprintln!("Failed to run shortcut: {}", e);
            }
        }
    }
}