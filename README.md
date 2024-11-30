[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/c2Cd-Xpe)
# Projexts

## Description

Projexts is a simple and efficient tool designed to help users easily manage shortcuts for running their frequently used programs and scripts. By allowing users to add, remove, and list shortcuts associated with specific commands, this tool enhances productivity and reduces the time spent navigating complex command-line inputs. The tool stores project names alongside their corresponding run commands in a lightweight configuration file, enabling users to access their shortcuts easily from the command line. 
 
## Installation

To compile and install the Command Line Shortcut Manager, ensure you have the Rust programming language installed on your system. You can follow the steps below:

1. Clone the repository:
   ```bash
   git clone https://github.com/rustvu-2024f/project-mariusschueller
   cd projexts
   ```

2. Compile the project using Cargo:
   ```bash
   cargo build --release
   ```

3. Move the compiled binary to a directory in your system's PATH for easier access:
   ```bash
   cp target/release/projexts /usr/local/bin/
   ```


## How to use

Once the Command Line Shortcut Manager is installed, you can start managing your shortcuts. Below are some common commands:

1. **Add a Shortcut**
   ```bash
   projexts add <name> -- <command> [extra_args...]
   ```
   Add a new shortcut with a project name and associated command. Optionally appending "[extra_args...]" extra parameters to be saved if desired. Local paths are also able to be saved.

2. **List Shortcuts**
   ```bash
   projexts list
   ```
   Display all stored shortcuts and their associated commands.

3. **Run a Shortcut**
   ```bash
   projexts run <name> -- [extra_args...]
   ```
   Execute the command associated with a given shortcut, optionally appending "-- [extra_args...]" for additional arguments.

4. **Update a Shortcut**
   ```bash
   projexts update <name> -- <new_command> [extra_args...]
   ```
   Modify the command of an existing shortcut. Can optionally append "-- [extra_args...]" for additional arguments.

5. **Remove a Shortcut**
   ```bash
   projexts remove <name>
   ```
   Delete a shortcut from the configuration file.

6. **Reset Shortcuts**
   ```bash
   projexts reset
   ```
   Delete the configuration file to clear all stored shortcuts.

7. **Open a Project Folder**
   ```bash
   projexts open <name>
   ```
   Open the directory associated with the specified shortcut.

8. **Open a Project File**
   ```bash
   projexts open-file <name>
   ```
   Open the first file found associated with the specified shortcut.

9. **Git Commit and Push**
   ```bash
   projexts git-push <name> <commit_message>
   ```
   Commit and push changes to a Git repository linked to the project shortcut.

By following these commands, users can efficiently manage their shortcuts, making it easy to run their preferred programs directly from the command line.

*Note that for multithreading can't be used when testing. Please the following command:
 ```bash
cargo test -- --test-threads=1
```
