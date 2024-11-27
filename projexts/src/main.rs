use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use serde::{Serialize, Deserialize};
use serde_json;
use std::path::Path;

/// Returns the path to the configuration file for storing shortcuts.
///
/// This function constructs the file path for the configuration file by using the user's home directory
/// and appending the filename `.projexts_config.json` to it. It leverages the `dirs` crate to determine
/// the home directory.
///
/// # Panics
/// This function will panic if the `dirs::home_dir()` function returns `None`, indicating that the home
/// directory could not be determined (e.g., in environments without a user home directory, such as some
/// containerized or certain restricted systems).
fn config_file_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".projexts_config.json")
}

/// Loads the list of shortcuts from the persistent storage file.
///
/// This function checks if the configuration file exists at the specified path. If the file does not
/// exist, it creates a new file with an empty JSON array (`[]`) as its content. After ensuring the
/// file exists, it reads the data from the file, deserializes it into a `Vec<Shortcut>`, and returns
/// the list of shortcuts.
///
/// # Errors
/// This function may return an error if:
/// - The configuration file cannot be read (e.g., due to I/O errors).
/// - The file content cannot be successfully deserialized into a `Vec<Shortcut>`.
/// - There is an error while creating the file if it doesn't exist.
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

/// Saves the given list of shortcuts to persistent storage.
///
/// This function serializes the provided vector of `Shortcut` objects into a JSON
/// format and writes it to the configuration file. If the operation is successful,
/// the updated list of shortcuts will be stored and available for future access.
///
/// # Parameters
/// - `shortcuts`: A slice of `Shortcut` objects that represents the list of shortcuts
///   to be saved. Each `Shortcut` contains a project name and the associated run command.
///
/// # Errors
/// This function will return an error if:
/// - The `serde_json::to_string_pretty` function fails to serialize the `shortcuts` vector.
/// - The `fs::write` function fails to write the serialized data to the storage file.
fn save_shortcuts(shortcuts: &[Shortcut]) -> io::Result<()>{
    let data = serde_json::to_string_pretty(shortcuts)?;
    fs::write(config_file_path(), data)?;
    Ok(())
}

/// Adds a new shortcut with the given name and command to the storage.
///
/// This function adds a new shortcut, consisting of a project name and a command, to the list of stored
/// shortcuts. It first validates that the command is not empty and then ensures that all paths in the
/// command are either absolute or can be converted to absolute paths. If any relative paths are provided,
/// they are converted to absolute paths using `fs::canonicalize()`. If a valid path is not found for any 
/// command component, an error is returned.
///
/// # Arguments
/// * `name` - The name of the project or shortcut.
/// * `command` - A vector of strings representing the command to run, where each string is a part of the command (e.g., executable name, arguments).
///
/// # Returns
/// * `Ok(())` if the shortcut is successfully added to the storage.
/// * `Err(io::Error)` if the command is empty, or if no valid paths are found in the command.
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
            absolute_command.push(cmd); 
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

/// Removes a shortcut with the given name from the storage.
///
/// This function searches for a shortcut with the specified `name` and removes it from the list of stored
/// shortcuts. If no shortcut with the given name is found, a message is printed indicating that the shortcut
/// does not exist. If the shortcut is successfully removed, the list of shortcuts is saved back to storage.
///
/// # Arguments
/// * `name` - The name of the project or shortcut to remove.
///
/// # Returns
/// * `Ok(())` if the shortcut is removed successfully or if no matching shortcut is found (in which case no changes are made).
/// * `Err(io::Error)` if an error occurs while loading or saving the shortcuts.
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

/// Lists all the stored shortcuts and their associated commands.
///
/// This function loads the list of shortcuts from storage and prints each shortcut's project name
/// along with the corresponding run command. If no shortcuts are found, a message indicating that
/// no shortcuts are available is printed.
///
/// # Returns
/// * `Ok(())` if the list of shortcuts is successfully retrieved and printed.
/// * `Err(io::Error)` if an error occurs while loading the shortcuts.
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

/// Opens the project folder associated with the given shortcut name.
///
/// This function searches for a shortcut with the specified name, retrieves the first command
/// from the shortcut's `run_command` (assumed to be the project folder path), and opens that folder
/// using the appropriate system file manager. If no valid shortcut is found or if there is an issue
/// with the folder path, an error message is printed.
///
/// # Arguments
/// * `name` - The name of the project whose folder is to be opened.
///
/// # Returns
/// * `Ok(())` if the folder is successfully opened.
/// * `Err(io::Error)` if an error occurs while retrieving the shortcut or opening the folder.
///
/// # Errors
/// The function will return an error if:
/// - No shortcut with the given name is found.
/// - The `run_command` for the shortcut is empty.
/// - The folder path is invalid or cannot be determined from the run command.
/// - The operating system is unsupported (other than Windows, macOS, or Linux).
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

/// Runs the command associated with a given shortcut, with optional additional arguments.
///
/// This function searches for a shortcut by name, retrieves the associated command, and runs it with
/// the stored arguments combined with any additional arguments provided by the user. The command is
/// executed and the function waits for it to complete before returning.
///
/// # Arguments
/// * `name` - The name of the project whose associated command is to be run.
/// * `extra_args` - A vector of extra arguments to append to the command’s stored arguments.
///
/// # Returns
/// * `Ok(())` if the command is executed successfully.
/// * `Err(io::Error)` if an error occurs while retrieving the shortcut or running the command.
///
/// # Errors
/// The function will return an error if:
/// - No shortcut with the given name is found.
/// - The `run_command` for the shortcut is empty.
/// - An error occurs when trying to spawn or wait for the command to finish.
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

/// Updates the command of an existing shortcut.
///
/// This function searches for a shortcut by its name and updates its associated command if found.
/// If a new command is provided, it replaces the existing command for that shortcut. If the shortcut
/// is found and updated successfully, the changes are saved to storage.
///
/// # Arguments
/// * `name` - The name of the shortcut to update.
/// * `new_command` - An optional vector of new command arguments. If `Some(command)` is provided, 
///   the command associated with the shortcut will be replaced with this new command. If `None` is 
///   provided, the command will not be changed.
///
/// # Returns
/// * `Ok(())` if the shortcut is found and updated successfully, and the changes are saved.
/// * `Err(io::Error)` if an error occurs while loading or saving the shortcuts, or if the shortcut 
///   with the given name is not found.
///
/// # Errors
/// The function will return an error if:
/// - No shortcut with the given name is found.
/// - An error occurs while saving the updated list of shortcuts to storage.
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

/// Opens a file from a shortcut's command list.
///
/// This function searches for a shortcut by its name and attempts to open each file path in the shortcut's
/// command list. The file paths are opened using the system's default file manager. The function will open
/// each file path as long as the path exists and is a valid file.
///
/// # Arguments
/// * `name` - The name of the shortcut whose command list will be used to find and open the file paths.
///
/// # Returns
/// * `Ok(())` if the file(s) were opened successfully.
/// * `Err(io::Error)` if an error occurs while loading the shortcuts, or if the shortcut with the given
///   name is not found, or if any file in the shortcut's command list cannot be opened.
///
/// # Errors
/// The function will return an error if:
/// - No shortcut with the given name is found.
/// - Any of the paths in the shortcut are invalid, do not exist, or are not files.
/// - The operating system is unsupported for file opening commands.
fn open_file_from_shortcut(name: &str) -> io::Result<()> {
    let shortcuts = load_shortcuts()?;
    if let Some(shortcut) = shortcuts.iter().find(|s| s.project_name == name) {
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

        for file_path in &shortcut.run_command {
            let path = Path::new(file_path);

            if path.exists() && path.is_file() {
                Command::new(open_command)
                    .arg(path)
                    .spawn()?
                    .wait()?; // Wait for the command to complete
                println!("Opening file: {:?}", file_path);
            } else {
                eprintln!("Error: '{}' does not exist or is not a file.", file_path);
            }
        }
    } else {
        eprintln!("Error: No shortcut found with name '{}'", name);
    }
    Ok(())
}

/// Commits and pushes changes to a Git repository using a shortcut's project directory.
///
/// This function finds the shortcut associated with the given `name`, navigates to the project directory
/// specified in the shortcut's `run_command`, and performs a `git add`, `git commit`, and `git push` with the
/// specified commit message.
///
/// # Arguments
/// * `name` - The name of the shortcut whose associated Git project will be used.
/// * `commit_message` - The commit message to use for the `git commit` command.
///
/// # Returns
/// * `Ok(())` if the Git operations (add, commit, push) were successful.
/// * `Err(io::Error)` if any error occurs during the Git operations, loading shortcuts, or if the shortcut
///   cannot be found.
///
/// # Errors
/// The function will return an error if:
/// - No shortcut with the given name is found.
/// - The directory from the shortcut's `run_command` cannot be determined or is invalid.
/// - Any of the Git commands (`git add`, `git commit`, `git push`) fail.
fn git_push(name: &str, commit_message: &str) -> io::Result<()> {
    let shortcuts = load_shortcuts()?;
    if let Some(shortcut) = shortcuts.iter().find(|s| s.project_name == name) {
        if let Some(first_command) = shortcut.run_command.first() {
            let path = Path::new(first_command);

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

            // Change to the directory
            std::env::set_current_dir(dir)?;

            // Add changes
            Command::new("git")
                .arg("add")
                .arg(".")
                .status()?;

            // Commit changes
            Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(commit_message)
                .status()?;

            // Push changes
            Command::new("git")
                .arg("push")
                .status()?;

            println!("Changes committed and pushed from directory {:?}", dir);
        } else {
            eprintln!("Error: Run command is empty for shortcut '{}'", name);
        }
    } else {
        eprintln!("Error: No shortcut found with name '{}'", name);
    }
    Ok(())
}

/// Represents a shortcut for a project, including the project's name and the command to run.
///
/// This struct is used to store and manage shortcuts for projects, where each shortcut has:
/// - `project_name`: The name of the project associated with the shortcut.
/// - `run_command`: A vector of strings representing the command and its arguments to execute the project.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Shortcut {
    /// The name of the project associated with the shortcut.
    project_name: String,
    
    /// The command (with its arguments) to run the project.
    run_command: Vec<String>,
}

/// A command-line interface (CLI) tool to manage project shortcuts.
///
/// This struct represents the root of the CLI and serves as an entry point for handling
/// various commands that interact with project shortcuts (e.g., adding, removing, listing shortcuts).
///
/// The CLI tool uses `clap` to parse commands and subcommands, providing a user-friendly way to interact
/// with the project management functionality.
#[derive(Parser)]
#[command(name = "projexts", about = "A CLI tool to manage project shortcuts")]
struct Cli {
    /// The subcommand to execute.
    /// 
    /// This field allows the user to specify which action to take. Each subcommand corresponds to a
    /// specific operation on the project shortcuts (e.g., adding, removing, listing shortcuts).
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
    Open {
        name: String,
    },
    /// Open a file from a shortcut
    OpenFile {
        /// Name of the project
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
    /// Update an existing shortcut
    Update {
        /// Name of the project
        name: String,
        /// Command to run the project (supports spaces and arguments)
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Add, commit, and push changes to git in directory of the shortcut
    GitPush {
        /// Name of the project
        name: String,
        /// Commit message
        commit_message: String,
    },
}

/// The main entry point for the `projexts` CLI tool.
///
/// This function parses the command-line arguments using `Cli::parse()` and dispatches the appropriate
/// subcommand based on the user's input. Each subcommand corresponds to a specific operation (such as adding, 
/// removing, or listing shortcuts), and the function handles any errors that occur during execution.
///
/// It performs the following tasks:
/// - Adds a new shortcut using the `add_shortcut` function.
/// - Removes a shortcut using the `remove_shortcut` function.
/// - Lists all shortcuts using the `list_shortcuts` function.
/// - Opens the project folder using the `open_project_folder` function.
/// - Opens a file from a shortcut using the `open_file_from_shortcut` function.
/// - Runs a shortcut's command using the `run_shortcut` function.
/// - Updates an existing shortcut using the `update_shortcut` function.
/// - Pushes changes to Git using the `git_push` function.
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
                eprintln!("Failed to open project folder: {}", e);
            }
        }
        Commands::OpenFile { name } => {
            if let Err(e) = open_file_from_shortcut(&name) {
                eprintln!("Failed to open file from shortcut: {}", e);
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
                eprintln!("Failed to update shortcut: {}", e);
            }
        }
        Commands::GitPush { name, commit_message } => {
            println!("Pushing changes with commit message: {}", commit_message);
            if let Err(e) = git_push(&name, &commit_message) {
                eprintln!("Failed to push changes: {}", e);
            }
        }
    }
}

// Testing Code
/////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_config_file_path() -> PathBuf {
        std::env::temp_dir().join(".projexts_config.json")
    }

    fn setup_test_env() -> io::Result<()> {
        let config_path = test_config_file_path();
        if config_path.exists() {
            fs::remove_file(&config_path)?;
        }
        Ok(())
    }

    fn load_shortcuts_for_test() -> io::Result<Vec<Shortcut>> {
        let path = test_config_file_path();
        if !path.exists() {
            fs::File::create(&path)?.write_all(b"[]")?;
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save_shortcuts_for_test(shortcuts: &[Shortcut]) -> io::Result<()> {
        let data = serde_json::to_string_pretty(shortcuts)?;
        fs::write(test_config_file_path(), data)?;
        Ok(())
    }

    #[test]
    fn test_add_shortcut() -> io::Result<()> {
        setup_test_env()?;

        let name = "TestProject";
        let command = vec!["echo".to_string(), "Hello, world!".to_string()];
        let mut shortcuts = load_shortcuts_for_test()?;

        shortcuts.push(Shortcut {
            project_name: name.to_string(),
            run_command: command.clone(),
        });

        save_shortcuts_for_test(&shortcuts)?;

        let loaded_shortcuts = load_shortcuts_for_test()?;
        assert_eq!(loaded_shortcuts.len(), 1);
        assert_eq!(loaded_shortcuts[0].project_name, name);
        assert_eq!(loaded_shortcuts[0].run_command, command);

        Ok(())
    }

    #[test]
    fn test_remove_shortcut() -> io::Result<()> {
        setup_test_env()?;

        let shortcuts = vec![Shortcut {
            project_name: "TestProject".to_string(),
            run_command: vec!["echo".to_string(), "Hello, world!".to_string()],
        }];
        save_shortcuts_for_test(&shortcuts)?;

        let mut loaded_shortcuts = load_shortcuts_for_test()?;
        loaded_shortcuts.retain(|s| s.project_name != "TestProject");
        save_shortcuts_for_test(&loaded_shortcuts)?;

        let updated_shortcuts = load_shortcuts_for_test()?;
        assert!(updated_shortcuts.is_empty());

        Ok(())
    }

    #[test]
    fn test_list_shortcuts() -> io::Result<()> {
        setup_test_env()?;

        let shortcuts = vec![
            Shortcut {
                project_name: "ProjectOne".to_string(),
                run_command: vec!["echo".to_string(), "One".to_string()],
            },
            Shortcut {
                project_name: "ProjectTwo".to_string(),
                run_command: vec!["echo".to_string(), "Two".to_string()],
            },
        ];
        save_shortcuts_for_test(&shortcuts)?;

        let output = std::panic::catch_unwind(|| list_shortcuts());
        assert!(output.is_ok());

        Ok(())
    }
}
