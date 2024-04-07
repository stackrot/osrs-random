use clap::{Parser, Subcommand};
use colored::*;
use prettytable::{Table, row};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::io::{self, Write};

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
        println!("{}", "OSRS Random Generator".bold().underline().cyan());
        println!("{}", "Please choose an option:".cyan());
        println!("1. Boss Chooser");
        println!("2. Skill Chooser");
        println!("3. Exit");
        print!("{}", "Enter your choice (1, 2, or 3): ".cyan());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
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

fn show_help() {
    clear_screen();
    println!("{}", "OSRS Random Generator Help:".cyan());
    println!("1. Boss Chooser - Randomly select a boss from various categories.");
    println!("2. Skill Chooser - Randomly select a skill to train.");
    println!("3. Exit - Exit the application.\n");
    println!("You can use the interactive menu or pass commands directly as arguments.");
    pause_before_clearing();
}

fn pause_before_clearing() {
    println!("\nPress enter to continue...");
    let _ = io::stdin().read_line(&mut String::new()).unwrap();
    clear_screen();
}

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn load_bosses() -> HashMap<&'static str, Vec<&'static str>> {
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
}

fn generate_skill() {
    let mut rng = rand::thread_rng();
    let skills = ["Attack", "Strength", "Defence", "Ranged", "Prayer", "Magic", "Hitpoints",
                  "Runecraft", "Crafting", "Mining", "Smithing", "Fishing", "Cooking", "Firemaking",
                  "Woodcutting", "Agility", "Herblore", "Thieving", "Fletching", "Slayer", "Farming",
                  "Construction", "Hunter"];
    let skill = skills.choose(&mut rng).unwrap();

    println!("\nRandomly selected skill to train:\n");
    let mut table = Table::new();
    table.add_row(row![skill.bold().green()]);
    table.printstd();
    pause_before_clearing();
}

fn generate_boss() {
    let categories = load_bosses();
    let keys: Vec<&str> = categories.keys().cloned().collect();
    
    println!("{}", "\nDo you want to exclude any categories? (yes/no)".cyan());
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    let mut exclusions = Vec::new();
    if choice.trim().eq_ignore_ascii_case("yes") {
        println!("{}", "\nEnter the numbers of categories you wish to exclude, separated by spaces (e.g., 1 3 5):".cyan());
        for (index, key) in keys.iter().enumerate() {
            println!("{}. {}", index + 1, key);
        }

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        exclusions = input.trim().split_whitespace()
            .filter_map(|num| num.parse::<usize>().ok())
            .filter(|&num| num > 0 && num <= keys.len())
            .collect();
    }

    let mut rng = rand::thread_rng();
    let filtered_keys_indices: Vec<usize> = keys.iter().enumerate()
        .filter_map(|(index, _)| {
            if !exclusions.contains(&(index + 1)) {
                Some(index)
            } else {
                None
            }
        })
        .collect();

    if filtered_keys_indices.is_empty() {
        println!("All categories have been excluded. No bosses available.");
        return;
    }

    let filtered_keys: Vec<&str> = filtered_keys_indices.iter()
        .map(|&index| keys[index])
        .collect();

    let category = filtered_keys.choose(&mut rng).unwrap();
    let bosses = categories.get(category).unwrap();
    let boss = bosses.choose(&mut rng).unwrap();

    println!();
    let mut table = Table::new();
    table.add_row(row!["Category", "Boss"]);
    table.add_row(row![category.bold().yellow(), boss.bold().green()]);
    table.printstd();
    pause_before_clearing();
}

