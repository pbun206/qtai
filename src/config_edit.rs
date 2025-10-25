use anyhow::*;
use colored::Colorize;
use std::{
    fs::{self, *},
    io::Write,
    path::*,
};
use toml_edit::{DocumentMut};

use crate::{collections::Collection, config::Config};

const COMPLETION_MESSAGE: &str = "Done (^-^)b";

pub fn add_item(
    collection_query: String,
    key: String,
    value: String,
    config_path: PathBuf,
    config: Config,
) -> Result<()> {
    let selected_collection = config.select_collections(&collection_query)?;
    if selected_collection.1.items.contains_key(&key) {
        Err(anyhow!("Collection already has key."))
    } else {
        let config_file = fs::read_to_string(&config_path)?;
        let mut doc = config_file
            .parse::<DocumentMut>()
            .expect("invalid document");
        doc["collections"][selected_collection.0][key] = value.into();
        write(config_path, doc.to_string())?;
        println!(
            "Key pair added to collection \"{}\".",
            selected_collection.0
        );
        println!("{}", COMPLETION_MESSAGE);
        Ok(())
    }
}

pub fn remove_item(query: &str, config_path: PathBuf, config: Config) -> Result<()> {
    let selected_item: (&str, (&str, &str)) = config.select_items(query)?;
    println!(
        "Deleting \"{}\": \"{}\" from collection \"{}\"",
        selected_item.1.0, selected_item.1.1, selected_item.0
    );
    let config_file = fs::read_to_string(&config_path)?;
    let mut doc = config_file
        .parse::<DocumentMut>()
        .expect("invalid document");
    doc["collections"][selected_item.0]
        .as_table_mut()
        .context("Trouble converting collection as a table")?
        .remove(selected_item.1.0);
    write(config_path, doc.to_string())?;
    println!("Item removed.");
    println!("{}", COMPLETION_MESSAGE);
    Ok(())
}

pub fn add_collection(name: String, config_path: PathBuf, config: Config) -> Result<()> {
    if config.collections.contains_key(&name) {
        Err(anyhow!("Config already has this key."))
    } else {
        // Add new collection to the end of the line.
        let mut config_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(config_path)
            .unwrap();

        // Add extra line at the end to give good spacing
        if let Err(e) = writeln!(config_file, "[collections.\"{}\"]\n", name) {
            eprintln!("Couldn't write to file: {}", e);
        }
        println!("Collection added.");
        println!("{}", COMPLETION_MESSAGE);
        Ok(())
    }
}

pub fn remove_collection(
    query: &str,
    config_path: PathBuf,
    config: Config,
    assume_yes: bool,
) -> Result<()> {
    let selected_collection: (&str, &Collection) = config.select_collections(query)?;
    println!("Found collection: \"{}\"", selected_collection.0.bold());
    crate::with_confirmation(
        assume_yes,
        "Are you sure? This cannot be undone.",
        Some(false),
        || {
            let config_file = fs::read_to_string(&config_path)?;
            let mut doc = config_file
                .parse::<DocumentMut>()
                .expect("invalid document");
            doc["collections"]
                .as_table_mut()
                .context("Trouble converting collection as a table")?
                .remove(selected_collection.0);
            write(config_path, doc.to_string())?;
            println!("Collection \"{}\" removed.", selected_collection.0);
            println!("{}", COMPLETION_MESSAGE);
            Ok(())
        },
        || Err(anyhow!("User changed their mind.")),
    )
}

/// Generates config file based on the path given
pub fn generate_config_file(config_path: &PathBuf) -> anyhow::Result<Config> {
    // Checks and creates parent directories
    match config_path.parent() {
        Some(d) => {
            if !d.is_dir() {
                create_dir_all(d)?;
                println!("Created parent directories.")
            }
        }
        // None equals root directory
        None => {}
    };

    // Creates a new config file
    let config = Config::template();
    write(config_path, toml::to_string_pretty(&config)?)?;
    let config_file = fs::read_to_string(&config_path)?;
    let mut doc = config_file
        .parse::<DocumentMut>()
        .expect("invalid document");
    // Ask toml_edit to do the last part in single quotes
    doc["default_runner"] = "'notify-send $1'".parse::<toml_edit::Item>().unwrap();
    write(config_path, doc.to_string())?;
    println!("Config generated (^-^)b");
    Ok(config)
}

/// Edits runner in config file, collection may or may not be specified
pub fn change_runner(
    new_runner: &str,
    collection_query: Option<String>,
    config_path: &PathBuf,
    config: &Config,
) -> Result<()> {
    match collection_query {
        Some(q) => {
            let selected_collection: (&str, &Collection) = config.select_collections(&q)?;
            println!("Found collection: \"{}\"", selected_collection.0.bold());
            let config_file = fs::read_to_string(&config_path)?;
            let mut doc = config_file
                .parse::<DocumentMut>()
                .expect("invalid document");
            // Ask toml_edit to do the last part in single quotes
            doc["collections"][selected_collection.0]["default_runner"] = format!("'{}'", new_runner)
                .parse::<toml_edit::Item>()
                .unwrap();
            write(config_path, doc.to_string())?;
            println!("{}", COMPLETION_MESSAGE);
            Ok(())
        }
        None => {
            let config_file = fs::read_to_string(&config_path)?;
            let mut doc = config_file
                .parse::<DocumentMut>()
                .expect("invalid document");
            println!("'{}'", new_runner);
            // Ask toml_edit to do the last part in single quotes
            doc["default_runner"] = format!("'{}'", new_runner)
                .parse::<toml_edit::Item>()
                .unwrap();
            write(config_path, doc.to_string())?;
            println!("{}", COMPLETION_MESSAGE);
            Ok(())
        }
    }
}

pub fn change_menu(new_menu: &str, config_path: &PathBuf, _: Config) -> Result<()> {
            let config_file = fs::read_to_string(&config_path)?;
            let mut doc = config_file
                .parse::<DocumentMut>()
                .expect("invalid document");
            // Ask toml_edit to do the last part in single quotes
            doc["default_menu"] = format!("'{}'", new_menu)
                .parse::<toml_edit::Item>()
                .unwrap();
            write(config_path, doc.to_string())?;
        println!("{}", COMPLETION_MESSAGE);
            Ok(())
}
