use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use serde::{Serialize, Deserialize};
use serde_json;
use std::path::Path;


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

fn add_shortcut(name: &str, command: Vec<String>) -> io::Result<()> {
    if command.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Command cannot be empty"));
    }

    // Convert relative paths to absolute paths and validate
    let mut absolute_command: Vec<String> = Vec::new();
    let mut valid_path_found = false;

    for cmd in command {
        let path = Path::new(&cmd);
        if path.is_absolute() {
            if path.exists() {
                absolute_command.push(cmd.clone());
                valid_path_found = true;
            }
        } else if let Ok(abs_path) = fs::canonicalize(path) {
            absolute_command.push(abs_path.to_string_lossy().to_string());
            valid_path_found = true;
        } else {
            absolute_command.push(cmd); // If invalid, keep the original (optional)
        }
    }

    if !valid_path_found {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No valid paths in the command"));
    }

    let mut shortcuts = load_shortcuts()?;
    shortcuts.push(Shortcut {
        project_name: name.to_string(),
        run_command: absolute_command,
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

    if shortcuts.is_empty() {
        println!("No shortcuts found.");
    } else {
        for shortcut in shortcuts {
            println!("{}: {:?}", shortcut.project_name, shortcut.run_command);
        }
    }
    Ok(())
}

fn open_project_folder(name: &str) -> io::Result<()> {
    let shortcuts = load_shortcuts()?;
    if let Some(shortcut) = shortcuts.iter().find(|s| s.project_name == name) {
        println!("Opening project folder for: {:?}", shortcut.project_name);

        if let Some(first_command) = shortcut.run_command.first() {
            let path = std::path::Path::new(first_command);

            // Get the directory of the path
            let dir = if path.is_dir() {
                path
            } else if let Some(parent) = path.parent() {
                parent
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Unable to determine directory from run command",
                ));
            };

            // Open the directory using system file manager
            let open_command = if cfg!(target_os = "windows") {
                "explorer"
            } else if cfg!(target_os = "macos") {
                "open"
            } else if cfg!(target_os = "linux") {
                "xdg-open"
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Unsupported operating system",
                ));
            };

            Command::new(open_command)
                .arg(dir)
                .spawn()?
                .wait()?; // Wait for the command to complete

        } else {
            eprintln!("Error: Run command is empty for project '{}'", name);
        }
    } else {
        eprintln!("Error: No shortcut found with name '{}'", name);
    }
    Ok(())
}


fn run_shortcut(name: &str, extra_args: Vec<String>) -> io::Result<()> {
    let shortcuts = load_shortcuts()?;
    if let Some(shortcut) = shortcuts.iter().find(|s| s.project_name == name) {
        println!("Running command: {:?}", shortcut.run_command);

        if let Some((command, args)) = shortcut.run_command.split_first() {
            // Combine stored args with extra args
            let combined_args: Vec<String> = args.iter().cloned().chain(extra_args).collect();

            Command::new(command)
                .args(&combined_args)
                .spawn()?
                .wait()?; // Wait for the command to complete
        } else {
            eprintln!("Error: Command for '{}' is empty.", name);
        }
    } else {
        eprintln!("Error: No shortcut found with name '{}'", name);
    }
    Ok(())
}

fn update_shortcut(name: &str, new_command: Option<Vec<String>>) -> io::Result<()> {
    let mut shortcuts = load_shortcuts()?;
    if let Some(shortcut) = shortcuts.iter_mut().find(|s| s.project_name == name) {
        if let Some(new_command) = new_command {
            shortcut.run_command = new_command;
        }
        save_shortcuts(&shortcuts)?;
        println!("Shortcut '{}' updated successfully.", name);
    } else {
        eprintln!("Error: No shortcut found with name '{}'", name);
    }
    Ok(())
}


#[derive(Serialize, Deserialize, Clone)]
struct Shortcut {
    project_name: String,
    run_command: Vec<String>,
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
        /// Command to run the project (supports spaces and arguments)
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Removes a shortcut
    Remove {
        /// Name of the project
        name: String,
    },
    /// List all shortcuts
    List,
    /// Opens the enclosed folder of the run command
    Open{
        name: String,
    },
    /// Run a shortcut by name
    Run {
        /// Name of the project to run
        name: String,
        /// Additional arguments to pass to the command
        #[arg(last = true)]
        extra_args: Vec<String>,
    },
    Update {
        /// Name of the project
        name: String,
        /// Command to run the project (supports spaces and arguments)
        #[arg(last = true)]
        command: Vec<String>,
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Add { name, command } => {
            println!("Adding shortcut: {} -> {:?}", name, command);
            if let Err(e) = add_shortcut(&name, command) {
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
        Commands::Open { name } => {
            if let Err(e) = open_project_folder(&name) {
                eprintln!("Failed to list shortcuts: {}", e);
            }
        }
        Commands::Run { name, extra_args } => {
            println!("Running shortcut '{}' with extra arguments: {:?}", name, extra_args);
            if let Err(e) = run_shortcut(&name, extra_args) {
                eprintln!("Failed to run shortcut: {}", e);
            }
        }

        Commands::Update { name, command } => {
            println!("Updating shortcut: {} -> {:?}", name, command);
            if let Err(e) = update_shortcut(&name, Some(command)) {
                eprintln!("Failed to add shortcut: {}", e);
            }
        }
    }
}