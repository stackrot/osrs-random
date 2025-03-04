use clap::{Parser, Subcommand};
use crossterm::style::{Attribute, Stylize};
use once_cell::sync::Lazy;
use prettytable::{row, Table};
use rand::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Write};

/// Static mapping of boss categories to their respective bosses
static BOSS_CATEGORIES: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        ("World Bosses", vec![
            "Barrows", "Scurrius", "Giant Mole", "Deranged Archaeologist", "DKs", "Sarachnis",
            "Perilous Moons", "Kalphite Queen", "Corporeal Beast", "Zulrah", "Vorkath", "Phantom Muspah",
            "Nightmare / Phosani's Nightmare", "Duke Sucellus", "The Leviathan", "The Whisperer", "Vardorvis", "Obor",
            "Bryophyta", "The Mimic", "Hespori", "Skotizo", "Amoxliatl", "The Hueycoatl", "Royal Titans"
        ]),
        ("God Wars", vec!["Kree'arra", "Zilyana", "Graardor", "K'ril", "Nex"]),
        ("Wilderness Bosses", vec![
            "Chaos Fanatic", "Crazy Archaeologist", "Scorpia", "King Black Dragon", 
            "Vet'ion / Calvar'ion", "Venenatis / Spindel", "Callisto / Artio",
            "Chaos Elemental"
        ]),
        ("Slayer Only Bosses", vec![
            "Grotesque Guardians", "Abyssal Sire", "Kraken", "Cerberus", "Thermonuclear Smoke Devil",
            "Alchemical Hydra", "Araxxor"
        ]),
        ("Minigame Bosses", vec!["Gauntlet", "TzTok-Jad", "TzKal-Zuk", "Sol Heredit"]),
        ("Skilling Bosses", vec!["Tempoross", "Wintertodt", "Zalcano"]),
        ("Raids", vec!["Chambers of Xeric", "Tombs of Amascut", "Theatre of Blood"]),
    ])
});

/// CLI configuration using clap
#[derive(Parser)]
#[clap(author = "stackrot", version = env!("CARGO_PKG_VERSION"), about = "OSRS Random Generator")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

/// Available commands for the CLI
#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Generate a random boss from various categories")]
    Boss,
    #[clap(about = "Generate a random skill to train")]
    Skill,
    #[clap(about = "Display help information")]
    Help,
    #[clap(about = "List all available bosses")]
    ListBosses,
    #[clap(about = "Display version information")]
    Version,
}

/// Main entry point for the application
fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Boss) => generate_boss(),
        Some(Commands::Skill) => generate_skill(),
        Some(Commands::Help) => show_help(),
        Some(Commands::ListBosses) => list_all_bosses(),
        Some(Commands::Version) => show_version(),
        None => interactive_menu(),
    }
}

/// Displays an interactive menu for the user to choose options
fn interactive_menu() {
    loop {
        clear_screen();
        println!("{}", "OSRS Random Generator".bold().attribute(Attribute::Underlined).cyan());
        println!("{}", "Please choose an option:".cyan());
        println!("1. Boss Chooser");
        println!("2. Skill Chooser");
        println!("3. List All Bosses");
        println!("4. Version Information");
        println!("5. Exit");
        print!("{}", "Enter your choice (1-5): ".cyan());
        io::stdout().flush().unwrap();

        let input = read_input();
        match input.trim() {
            "1" => generate_boss(),
            "2" => generate_skill(),
            "3" => list_all_bosses(),
            "4" => {
                clear_screen();
                show_version();
                pause_before_clearing();
            },
            "5" => {
                clear_screen();
                println!("Thank you for using the OSRS Random Generator!");
                break;
            }
            _ => {
                clear_screen();
                println!("{}", "Invalid option. Please try again.".red());
                pause_before_clearing();
            }
        }
    }
}

/// Reads a line of input from the user
fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

/// Pauses execution until the user presses enter, then clears the screen
fn pause_before_clearing() {
    println!("\nPress enter to continue...");
    let _ = io::stdin().read_line(&mut String::new()).unwrap();
    clear_screen();
}

/// Clears the terminal screen based on the operating system
fn clear_screen() {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "cls"])
            .status()
            .unwrap();
    } else {
        std::process::Command::new("clear").status().unwrap();
    }
}

/// Displays help information about the application
fn show_help() {
    clear_screen();
    println!("{}", "OSRS Random Generator Help:".cyan());
    println!("1. Boss Chooser - Randomly select a boss from various categories.");
    println!("2. Skill Chooser - Randomly select a skill to train.");
    println!("3. List All Bosses - Display all available bosses by category.");
    println!("4. Exit - Exit the application.\n");
    pause_before_clearing();
}

/// Displays the current version of the application
fn show_version() {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("OSRS Random Generator v{}", current_version);
    
    // Check for updates
    match check_for_updates(current_version) {
        Ok(has_update) => {
            if has_update {
                println!("\n{}", "A newer version is available!".yellow().bold());
                println!("{}", "Visit https://github.com/stackrot/osrs-random/releases to download the latest version.".yellow());
            } else {
                println!("\n{}", "You are using the latest version.".green());
            }
        },
        Err(e) => {
            println!("\n{}", "Could not check for updates.".red());
            println!("{}", format!("Error: {}", e).red());
        }
    }
}

/// GitHub release information
#[derive(Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
}

/// Checks if a newer version is available on GitHub
fn check_for_updates(current_version: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // GitHub API URL for releases
    let url = "https://api.github.com/repos/stackrot/osrs-random/releases/latest";
    
    // Create a client with a custom user agent
    let client = reqwest::blocking::Client::builder()
        .user_agent("osrs-random-version-checker")
        .build()?;
    
    // Make the request
    let response = client.get(url).send()?;
    
    // Check if the request was successful
    if response.status().is_success() {
        // Parse the response
        let release: GitHubRelease = response.json()?;
        
        // Extract version from tag (remove 'v' prefix if present)
        let latest_version = release.tag_name.trim_start_matches('v');
        
        // Special case: GitHub releases use timestamp-based versioning (e.g., 20250304093829)
        // while the package uses semantic versioning (e.g., 1.0.0)
        // We'll consider them equivalent for now
        if latest_version.len() > 8 && latest_version.chars().all(|c| c.is_digit(10)) {
            // This is a timestamp-based version, not a semantic version
            // For now, we'll consider the user to be up-to-date
            return Ok(false);
        }
        
        // Compare versions (simple string comparison)
        Ok(latest_version != current_version)
    } else {
        Err(format!("Failed to fetch latest release: HTTP {}", response.status()).into())
    }
}

/// Generates a random skill for the user to train
fn generate_skill() {
    let skills = [
        "Attack", "Strength", "Defence", "Ranged", "Prayer", "Magic", "Hitpoints",
        "Runecraft", "Crafting", "Mining", "Smithing", "Fishing", "Cooking", "Firemaking",
        "Woodcutting", "Agility", "Herblore", "Thieving", "Fletching", "Slayer", "Farming",
        "Construction", "Hunter"
    ];
    let skill = skills.choose(&mut rand::thread_rng()).unwrap();

    println!("\nRandomly selected skill to train:\n");
    let mut table = Table::new();
    table.add_row(row![skill.bold().green()]);
    table.printstd();
    pause_before_clearing();
}

/// Generates a random boss for the user to fight
/// 
/// Allows the user to exclude certain categories of bosses
fn generate_boss() {
    let keys: Vec<&str> = BOSS_CATEGORIES.keys().cloned().collect();
    println!("{}", "\nDo you want to exclude any categories? (yes/no)".cyan());
    let choice = read_input();

    let mut exclusions = Vec::new();
    if choice.eq_ignore_ascii_case("yes") {
        println!("{}", "\nEnter the numbers of categories you wish to exclude, separated by spaces:".cyan());
        for (index, key) in keys.iter().enumerate() {
            println!("{}. {}", index + 1, key);
        }

        let input = read_input();
        exclusions = input.split_whitespace()
            .filter_map(|num| num.parse::<usize>().ok())
            .filter(|&num| num > 0 && num <= keys.len())
            .collect();
    }

    let filtered_keys: Vec<&str> = if exclusions.is_empty() {
        keys
    } else {
        keys.into_iter().enumerate()
            .filter(|(i, _)| !exclusions.contains(&(i + 1)))
            .map(|(_, k)| k)
            .collect()
    };

    if filtered_keys.is_empty() {
        println!("All categories have been excluded. No bosses available.");
        return;
    }

    let category = filtered_keys.choose(&mut rand::thread_rng()).unwrap();
    let bosses = BOSS_CATEGORIES.get(category).unwrap();
    let boss = bosses.choose(&mut rand::thread_rng()).unwrap();

    println!();
    let mut table = Table::new();
    table.add_row(row!["Category", "Boss"]);
    table.add_row(row![category.bold().yellow(), boss.bold().green()]);
    table.printstd();
    pause_before_clearing();
}

/// Lists all available bosses organized by category
/// 
/// Also provides information about reporting missing bosses
fn list_all_bosses() {
    clear_screen();
    println!("{}", "All Available Bosses:".bold().attribute(Attribute::Underlined).cyan());
    println!();
    
    // Use simple text-based formatting instead of tables
    for (category, bosses) in BOSS_CATEGORIES.iter() {
        // Print category header
        println!("{}: ", category.bold().yellow());
        
        // Print bosses with proper wrapping
        let mut line = String::new();
        let max_line_length = 80;
        
        for boss in bosses {
            // If adding this boss would make the line too long, print the current line and start a new one
            if line.len() + boss.len() + 2 > max_line_length && !line.is_empty() {
                println!("  {}", line);
                line.clear();
            }
            
            // Add the boss to the current line
            if line.is_empty() {
                line.push_str(boss);
            } else {
                line.push_str(", ");
                line.push_str(boss);
            }
        }
        
        // Print any remaining bosses
        if !line.is_empty() {
            println!("  {}", line);
        }
        
        println!(); // Add a blank line between categories
    }
    
    println!();
    println!("{}", "Missing a boss? Please report it at:".cyan());
    println!("{}", "https://github.com/stackrot/osrs-random/issues".underlined().cyan());
    
    pause_before_clearing();
}
