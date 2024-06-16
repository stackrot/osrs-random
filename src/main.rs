use clap::{Parser, Subcommand};
use crossterm::style::{Attribute, Stylize};
use prettytable::{Table, row};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::io::{self, Write};
use once_cell::sync::Lazy;

static BOSS_CATEGORIES: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        ("World Bosses", vec![
            "Barrows", "Scurrius", "Giant Mole", "Deranged Archaeologist", "DKs", "Sarachnis",
            "Perilous Moons", "Kalphite Queen", "Corporeal Beast", "Zulrah", "Vorkath", "Phantom Muspah",
            "Nightmare", "Duke Sucellus", "The Leviathan", "The Whisperer", "Vardorvis", "Obor",
            "Bryophyta", "The Mimic", "Hespori", "Skotizo"
        ]),
        ("God Wars", vec!["Kree'arra", "Zilyana", "Graador", "K'ril", "Nex"]),
        ("Wilderness Bosses", vec![
            "Chaos Fanatic", "Crazy Archaeologist", "Scorpia", "King Black Dragon", "Calvar'ion",
            "Chaos Elemental", "Vet'ion", "Venenatis", "Callisto"
        ]),
        ("Slayer Only Bosses", vec![
            "Grotesque Guardians", "Abyssal Sire", "Kraken", "Cerberus", "Thermonuclear Smoke Devil",
            "Alchemical Hydra"
        ]),
        ("Minigame Bosses", vec!["Gauntlet", "TzTok-Jad", "TzKal-Zuk", "Sol Heredit"]),
        ("Skilling Bosses", vec!["Tempoross", "Wintertodt", "Zalcano"]),
        ("Raids", vec!["Chambers of Xeric", "Tombs of Amascut", "Theatre of Blood"]),
    ])
});

#[derive(Parser)]
#[clap(author = "stackrot", version = "1.0", about = "OSRS Random Generator")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Generate a random boss from various categories")]
    Boss,
    #[clap(about = "Generate a random skill to train")]
    Skill,
    #[clap(about = "Display help information")]
    Help,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Boss) => generate_boss(),
        Some(Commands::Skill) => generate_skill(),
        Some(Commands::Help) => show_help(),
        None => interactive_menu(),
    }
}

fn interactive_menu() {
    loop {
        clear_screen();
        println!("{}", "OSRS Random Generator".bold().attribute(Attribute::Underlined).cyan());
        println!("{}", "Please choose an option:".cyan());
        println!("1. Boss Chooser");
        println!("2. Skill Chooser");
        println!("3. Exit");
        print!("{}", "Enter your choice (1, 2, or 3): ".cyan());
        io::stdout().flush().unwrap();

        let input = read_input();
        match input.as_str() {
            "1" => generate_boss(),
            "2" => generate_skill(),
            "3" => return,
            _ => {
                println!("{}", "Invalid choice, please try again.".red());
                pause_before_clearing();
            },
        }
    }
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn pause_before_clearing() {
    println!("\nPress enter to continue...");
    let _ = io::stdin().read_line(&mut String::new()).unwrap();
    clear_screen();
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .unwrap();
    } else {
        std::process::Command::new("clear").status().unwrap();
    }
}

fn show_help() {
    clear_screen();
    println!("{}", "OSRS Random Generator Help:".cyan());
    println!("1. Boss Chooser - Randomly select a boss from various categories.");
    println!("2. Skill Chooser - Randomly select a skill to train.");
    println!("3. Exit - Exit the application.\n");
    pause_before_clearing();
}

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
