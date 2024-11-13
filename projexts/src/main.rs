use clap::Parser;
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use std::io::{self, Write};
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

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    action: String,
    name: Option<String>,
    command: Option<String>,
}

fn main() {
    let args = Cli::parse();

    match args.action.as_str() {
        "add" => {
            if let (Some(name), Some(command)) = (args.name.as_deref(), args.command.as_deref()) {
                println!("Adding shortcut: {} -> {}", name, command);
                if let Err(e) = add_shortcut(name, command) {
                    eprintln!("Failed to add shortcut: {}", e);
                }
            } else {
                println!("Error: 'add' action requires both <name> and <command>");
            }
        }
        "list" => {
            if let Err(e) = list_shortcuts() {
                eprintln!("Failed to list shortcuts: {}", e);
            }
        }
        "run" => {
            if let Some(name) = args.name.as_deref() {
                if let Err(e) = run_shortcut(name) {
                    eprintln!("Failed to run shortcut: {}", e);
                }
            } else {
                println!("Error: 'run' action requires <name>");
            }
        }
        _ => {
            println!("Error: unknown action '{}'. type projexts help to view commands", args.action);
        }
    }
}

