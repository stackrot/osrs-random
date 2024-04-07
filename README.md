# OSRS Random Generator

## Overview
The OSRS Random Generator is a command-line tool designed to help OSRS players randomly select bosses and skills to focus on. 

## Features
- **Boss Chooser**: Randomly selects a boss from various categories within OSRS.
- **Skill Chooser**: Randomly picks a skill for training, helping you decide what to level up next.

## Installation

### Prerequisites
Before you can use the OSRS Random Generator, you need to have Rust installed on your system. Rust provides the cargo package manager, which is essential for building and running Rust applications.

### Installing Rust
To install Rust, run the following command in your terminal:

- **Windows**:
  Install Rust using the Rustup-init executable:
  1. Download the `rustup-init.exe` using the [official Rust instructions](https://forge.rust-lang.org/infra/other-installation-methods.html).
  2. Run the downloaded executable and follow the on-screen instructions to install Rust.
  3. After installation, restart your command prompt to ensure the Rust binaries are in your PATH.

- **Linux/Mac**:
Open a terminal and execute:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

This command will download a script and start the installation of the Rust toolchain, which includes `cargo`, `rustc`, and other standard tools.

### Cloning and Building the Application
1. Clone the repository: `git clone https://github.com/stackrot/osrs-random.git`

2. Navigate to the project directory: `cd osrs-random`

3. Build the application using Cargo: `cargo build --release`

4. Run the application: `cargo run`
    4.1. Alternatively, you can run the compiled binary directly: `./target/release/osrs-random`
