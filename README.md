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

1. **Add a Shortcut:**
   ```bash
   projexts add <project-name> "<run-command>"
   ```
   Example:
   ```bash
   projexts add my_script "python3 my_script.py"
   ```

2. **Remove a Shortcut:**
   ```bash
   projexts remove <project-name>
   ```
   Example:
   ```bash
   projexts remove my_script
   ```

3. **List Shortcuts:**
   ```bash
   projexts list
   ```
   This command displays all stored project names and their associated commands.

4. **Run a Shortcut:**
   ```bash
   projexts run <project-name>
   ```
   Example:
   ```bash
   projexts run my_script
   ```

By following these commands, users can efficiently manage their shortcuts, making it easy to run their preferred programs directly from the command line.