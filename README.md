# OSRS Random Generator

## Overview
The OSRS Random Generator is a command-line tool designed to help OSRS players randomly select bosses and skills to focus on.

## Features
- **Boss Chooser**: Randomly selects a boss from various categories within OSRS.
- **Skill Chooser**: Randomly picks a skill for training, helping you decide what to level up next.

## Installation

You can download the latest release of the OSRS Random Generator for both Linux and Windows from the [Releases](https://github.com/stackrot/osrs-random/releases) page.

### Download and Run

#### Linux
1. Download the latest `osrs-random-linux.zip` from the [Releases](https://github.com/stackrot/osrs-random/releases) page.
2. Unzip the file:
    ```sh
    unzip osrs-random-linux.zip
    ```
3. Make the binary executable:
    ```sh
    chmod +x osrs-random
    ```
4. Run the application:
    ```sh
    ./osrs-random
    ```

#### Windows
1. Download the latest `osrs-random-windows.zip` from the [Releases](https://github.com/stackrot/osrs-random/releases) page.
2. Unzip the file.
3. Run the application by double-clicking `osrs-random.exe` or executing it from the command prompt:
    ```sh
    osrs-random.exe
    ```

## Usage

### Boss Chooser
To randomly select a boss from various categories:
```sh
osrs-random boss
```

### Skill Chooser
To randomly pick a skill to train:
```sh
osrs-random skill
```

### Interactive Menu
Run the application without arguments to use the interactive menu:
```sh
osrs-random
```

## Suggestions and Contributions

If you have any suggestions, feature requests, or would like to contribute, please open an issue or submit a pull request on the [GitHub repository](https://github.com/stackrot/osrs-random).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
