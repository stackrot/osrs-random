mod catalog;
mod net;
mod update;

use anyhow::{bail, Context, Result};
use catalog::Catalog;
use clap::{CommandFactory, Parser, Subcommand};
use crossterm::cursor::MoveTo;
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType};
use rand::seq::IndexedRandom;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

#[derive(Parser)]
#[command(author, version, about = "OSRS Random Generator")]
struct Cli {
    /// Use cached or bundled boss and skill data without network requests
    #[arg(long, global = true)]
    offline: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Choose a random boss
    Boss {
        /// Exclude a category by name (repeat for multiple categories)
        #[arg(long = "exclude")]
        exclusions: Vec<String>,
    },
    /// Choose a random skill to train
    Skill,
    /// List all bosses by category
    ListBosses,
    /// List all skills
    ListSkills,
    /// Refresh the cached boss and skill lists
    RefreshData,
    /// Display the installed version and release tag
    Version,
    /// Check for and install the latest release
    Update {
        /// Check for an update without installing it
        #[arg(long)]
        check: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        eprintln!("{error:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Version) => show_version(),
        Some(Commands::Update { check }) => {
            if cli.offline {
                bail!("Updating requires a network connection; remove --offline");
            }
            update_command(check)?;
        }
        Some(Commands::RefreshData) => {
            if cli.offline {
                bail!("Refreshing data requires a network connection; remove --offline");
            }
            let catalog = catalog::load(true, false)?;
            println!(
                "Refreshed {} bosses and {} skills.",
                catalog.bosses.values().map(Vec::len).sum::<usize>(),
                catalog.skills.len()
            );
        }
        Some(command) => {
            let catalog = catalog::load(false, cli.offline)?;
            match command {
                Commands::Boss { exclusions } => generate_boss(&catalog, &exclusions)?,
                Commands::Skill => generate_skill(&catalog)?,
                Commands::ListBosses => list_bosses(&catalog),
                Commands::ListSkills => println!("{}", catalog.skills.join(", ")),
                _ => unreachable!(),
            }
        }
        None if io::stdin().is_terminal() && io::stdout().is_terminal() => {
            interactive_menu(cli.offline)?;
        }
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }
    Ok(())
}

fn show_version() {
    println!("OSRS Random Generator v{}", env!("CARGO_PKG_VERSION"));
    if let Some(tag) = update::RELEASE_TAG {
        println!("Release: {tag}");
    }
}

fn update_command(check: bool) -> Result<bool> {
    match update::available()? {
        Some(release) if check => println!(
            "{} is available. Run 'osrs-random update' to install it.",
            release.tag_name
        ),
        Some(release) => {
            update::install(&release)?;
            return Ok(true);
        }
        None => println!("You are using the latest version."),
    }
    Ok(false)
}

fn offer_update() -> Result<bool> {
    match update::available() {
        Ok(Some(release)) => {
            println!("{} is available. Install it now? [y/N]", release.tag_name);
            if read_input()?
                .is_some_and(|input| matches!(input.to_ascii_lowercase().as_str(), "y" | "yes"))
            {
                update::install(&release)?;
                pause()?;
                return Ok(true);
            }
        }
        Ok(None) => {}
        Err(error) => eprintln!("Could not check for updates: {error}"),
    }
    Ok(false)
}

fn interactive_menu(offline: bool) -> Result<()> {
    clear_screen()?;
    if !offline && offer_update()? {
        return Ok(());
    }
    let mut catalog = None;
    loop {
        println!("{}", "OSRS Random Generator".bold().underlined().cyan());
        println!("1. Boss Chooser\n2. Skill Chooser\n3. List All Bosses");
        println!("4. Version Information\n5. Exit\n6. Update Application\n7. Refresh Bosses and Skills\n8. List All Skills");
        print!("Enter your choice (1-8): ");
        io::stdout().flush()?;
        let Some(input) = read_input()? else {
            return Ok(());
        };
        if input == "5" {
            return Ok(());
        }
        clear_screen()?;
        let result = menu_action(&input, offline, &mut catalog);
        match result {
            Ok(true) => {
                pause()?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => eprintln!("{error:#}"),
        }
        if !pause()? {
            return Ok(());
        }
        clear_screen()?;
    }
}

fn menu_action(input: &str, offline: bool, catalog: &mut Option<Catalog>) -> Result<bool> {
    match input {
        "4" => show_version(),
        "6" if !offline => return update_command(false),
        "7" if !offline => {
            *catalog = Some(catalog::load(true, false)?);
            println!("Refreshed boss and skill data.");
        }
        "6" | "7" => bail!("This option requires a network connection"),
        "1" | "2" | "3" | "8" => {
            if catalog.is_none() {
                *catalog = Some(catalog::load(false, offline)?);
            }
            let catalog = catalog.as_ref().context("Missing catalogue")?;
            match input {
                "1" => {
                    if let Some(exclusions) = read_exclusions(catalog)? {
                        generate_boss(catalog, &exclusions)?;
                    }
                }
                "2" => generate_skill(catalog)?,
                "3" => list_bosses(catalog),
                _ => println!("{}", catalog.skills.join(", ")),
            }
        }
        _ => println!("Invalid option. Please try again."),
    }
    Ok(false)
}

fn read_input() -> Result<Option<String>> {
    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 {
        return Ok(None);
    }
    Ok(Some(input.trim().to_owned()))
}

fn pause() -> Result<bool> {
    println!("\nPress enter to continue...");
    Ok(read_input()?.is_some())
}

fn clear_screen() -> Result<()> {
    crossterm::execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

fn read_exclusions(catalog: &Catalog) -> Result<Option<Vec<String>>> {
    let categories: Vec<_> = catalog.bosses.keys().collect();
    for (index, category) in categories.iter().enumerate() {
        println!("{}. {category}", index + 1);
    }
    println!("Enter category numbers to exclude, separated by spaces (enter for none):");
    let Some(input) = read_input()? else {
        return Ok(None);
    };
    let mut exclusions = Vec::new();
    for number in input.split_whitespace() {
        let index = number.parse::<usize>().context("Invalid category number")?;
        let category = index
            .checked_sub(1)
            .and_then(|index| categories.get(index))
            .context("Category number is out of range")?;
        exclusions.push((*category).clone());
    }
    Ok(Some(exclusions))
}

fn generate_skill(catalog: &Catalog) -> Result<()> {
    let skill = catalog
        .skills
        .choose(&mut rand::rng())
        .context("No skills available")?;
    println!("{skill}");
    Ok(())
}

fn generate_boss(catalog: &Catalog, exclusions: &[String]) -> Result<()> {
    for excluded in exclusions {
        if !catalog
            .bosses
            .keys()
            .any(|name| name.eq_ignore_ascii_case(excluded))
        {
            bail!("Unknown category '{excluded}'; see 'osrs-random list-bosses'");
        }
    }
    let categories: Vec<_> = catalog
        .bosses
        .iter()
        .filter(|(name, _)| {
            !exclusions
                .iter()
                .any(|excluded| name.eq_ignore_ascii_case(excluded))
        })
        .collect();
    let (category, bosses) = categories
        .choose(&mut rand::rng())
        .context("All categories have been excluded. No bosses available")?;
    let boss = bosses
        .choose(&mut rand::rng())
        .context("No bosses in category")?;
    println!("{category}: {boss}");
    Ok(())
}

fn list_bosses(catalog: &Catalog) {
    for (category, bosses) in &catalog.bosses {
        println!("{category}:");
        for boss in bosses {
            println!("  {boss}");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_definitions_are_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn exclusions_reject_typos_and_empty_selection() {
        let catalog = Catalog {
            bosses: [("World bosses".into(), vec!["Scurrius".into()])].into(),
            skills: vec!["Sailing".into()],
        };
        assert!(generate_boss(&catalog, &["typo".into()]).is_err());
        assert!(generate_boss(&catalog, &["WORLD BOSSES".into()]).is_err());
        assert!(generate_boss(&catalog, &[]).is_ok());
    }
}
