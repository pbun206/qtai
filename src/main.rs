pub mod collections;
pub mod config;
pub mod config_edit;
pub mod run;

use anyhow::*;
use clap::*;
use colored::*;
use dialoguer::*;
use figment::providers::*;
use figment::*;
use std::path::*;

use crate::config::Config;
use crate::config_edit::*;

/// Store the CLI subcommand
#[derive(Parser)]
#[command(version, about = "Run dmenu with configured items and runners", long_about = None)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
    #[arg(short = 'y', long, help = "Assume yes during confirmations")]
    assume_yes: bool,
    #[arg(short = 'c', long, help = "Custom config path")]
    config: Option<PathBuf>,
}

/// Store various CLI subcommands
#[derive(Subcommand, PartialEq)]
enum Subcommands {
    #[command(alias = "r", about = "Run dmenu application")]
    Run {
        #[arg(short = 'd', long, help = "Dmenu application. Default is \"dmenu\".")]
        dmenu: Option<String>,
        #[arg(
            short = 's',
            long,
            help = "Make collection more selective, only filtering collections with the exact same name"
        )]
        selective: bool,
        #[arg(short = 'r', long, help = "Command to run from item.")]
        runner: Option<String>,
        #[arg(help = "Collections to input")]
        collection_input: Vec<String>,
    },
    #[command(alias = "t", about = "Run within terminal")]
    TerminalRun {
        #[arg(short = 'r', long, help = "Command to run from item.")]
        runner: Option<String>,
        #[arg(
            short = 's',
            long,
            help = "Make collection more selective, only filtering collections with the exact same name"
        )]
        selective: bool,
        #[arg(help = "Collections to input")]
        collection_input: Vec<String>,
    },
    #[command(alias = "a", about = "Adds an item into a collection")]
    AddItem {
        #[arg(help = "Key to add.")]
        key: String,
        #[arg(help = "Value linked to the key to add.")]
        value: String,
        #[arg(help = "Determine what collection to edit.")]
        collection_query: String,
    },
    #[command(alias = "ri", about = "Removes an item from a config.")]
    RemoveItem { query: String },
    #[command(alias = "ac", about = "Adds a collection into a config.")]
    AddCollection {
        #[arg(help = "Collection name to add.")]
        name: String,
    },
    #[command(alias = "rc", about = "Remove a collection from the config.")]
    RemoveCollection { query: String },
    #[command(alias = "l", about = "Lists items from a config.")]
    List {
        #[arg(help = "Collections to list. Default is all.")]
        collections: Vec<String>,
        #[arg(
            short = 's',
            long,
            help = "Make collection more selective, only filtering collections with the exact same name"
        )]
        selective: bool,
    },
    #[command(alias = "s", about = "Search an item from a config.")]
    Search { query: String },
    #[command(alias = "cr", about = "Alter the default runner.")]
    ChangeRunner{ new_runner: String,
        #[arg(short = 'q', help = "Collection to change runner. Default is changing global runner.")]
        collection_query: Option<String>,
    },

    #[command(alias = "cm", about = "Alter the default menu for qtai run.")]
    ChangeMenu{
        new_menu: String,
    },

    #[command(alias = "gcf", about = "Generates config file.")]
    GenerateConfigFile,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or(
        dirs::config_dir()
            .context("Config directory not found.")?
            .join("qtai")
            .join("qtai.toml"),
    );
    let config = determine_config(&config_path, &cli.subcommand, cli.assume_yes)?;

    // Run subcommand
    match cli.subcommand {
        Subcommands::Run {
                    collection_input,
                    runner,
                    dmenu,
                    selective,
                } => crate::run::run(dmenu, &collection_input,runner, &config, cli.assume_yes, selective),
        Subcommands::TerminalRun {
                    collection_input,
                    runner,
                    selective,
                } => crate::run::terminal_run(
                    &collection_input,
                    runner,
                    &config,
                    cli.assume_yes,
                    selective,
                ),
        Subcommands::GenerateConfigFile {} => Ok(()),
        Subcommands::AddItem {
                    collection_query,
                    key,
                    value,
                } => add_item(collection_query, key, value, config_path, config),
        Subcommands::RemoveItem { query } => remove_item(&query, config_path, config),
        Subcommands::AddCollection { name } => add_collection(name, config_path, config),
        Subcommands::RemoveCollection { query } => {
                    remove_collection(&query, config_path, config, cli.assume_yes)
                }
        Subcommands::List {
                    collections,
                    selective,
                } => config.list_collections(&collections, selective),
        Subcommands::Search { query } => config.search_items(&query),
        Subcommands::ChangeRunner { new_runner, collection_query } => change_runner(&new_runner, collection_query, &config_path, &config),
        Subcommands::ChangeMenu { new_menu } => change_menu(&new_menu, &config_path, config),
    }
}

/// Try to find a config file. If generate config command or config file is not found,
/// then make a basic config file template.
fn determine_config(
    config_path: &PathBuf,
    subcommand: &Subcommands,
    assume_yes: bool,
) -> anyhow::Result<Config> {
    let result = Figment::new()
        .merge(Toml::file(&config_path))
        .extract::<Config>();
    match result {
        anyhow::Result::Ok(c) => {
            if &Subcommands::GenerateConfigFile == subcommand {
                // Checks if preexisting file exist and askes for overwritting confirmation
                println!("Existing config file will be overwritten.");
                with_confirmation(
                    assume_yes,
                    "Is this okay?",
                    None,
                    || {
                        println!("Config file is overwritten.");
                        generate_config_file(config_path)
                    },
                    || {
                        println!("No changes are written.");
                        Ok(Config::default())
                    },
                )
            } else {
                Ok(c)
            }
        }
        Err(e) => {
            // We will try to do a little debugging for them.
            if &Subcommands::GenerateConfigFile != subcommand && !config_path.is_file() {
                println!("Cannot find config file.");
            }
            // If no file or asked to, we will generate a config file.
            if &Subcommands::GenerateConfigFile == subcommand || !config_path.is_file() {
                println!("We will generate the config automatically.");
                with_confirmation(
                    assume_yes,
                    "Is this okay?",
                    Some(true),
                    || {
                        println!("Writing to config file.");
                        generate_config_file(config_path)
                    },
                    || {
                        println!("No changes are written.");
                        anyhow::Result::Err(anyhow!("No config file."))
                    },
                )
            } else {
                // When there is a config file but Figment errors out, there is something wrong.
                println!("{}", format!("{}", e).red());
                return anyhow::Result::Err(anyhow!(
                    "Something went wrong from reading config file. Make sure to have default runner and default menu set on the top level."
                ));
            }
        }
    }
}

/// Run a series of functions based on confirmation.
pub fn with_confirmation<F, T, G>(
    assume_yes: bool,
    prompt: &str,
    is_default: Option<bool>,
    yes_function: F,
    no_function: G,
) -> T
where
    F: FnOnce() -> T,
    G: FnOnce() -> T,
{
    if !assume_yes {
        let confirmation = match is_default {
            Some(default) => Confirm::new().default(default),
            None => Confirm::new(),
        }
        .with_prompt(prompt)
        .interact()
        .unwrap();
        if confirmation {
            yes_function()
        } else {
            no_function()
        }
    } else {
        yes_function()
    }
}
