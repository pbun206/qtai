use anyhow::*;
use dialoguer::Select;
use indexmap::IndexMap;
use std::process::{Command, Stdio};

use crate::config::Config;

///Run command into a dmenu and runs the output based on config
pub fn run(
    menu_option: Option<String>,
    collections_input: &[String],
    runner: Option<String>,
    config: &Config,
    _assume_yes: bool,
    selective: bool,
) -> Result<()> {
    let menu = match menu_option {
        Some(x) => x,
        None => config.default_menu.clone(),
    };
    // In the format of collection, keypair
    let items: Vec<(&str, (&str, &str))> = config
        .filter_collections(collections_input, selective)
        .iter()
        .flat_map(|i| {
            i.1.items
                .iter()
                .map(|j| (*i.0, (j.0.as_str(), j.1.as_str())))
        })
        .collect();
    if items.len() == 0 {
        return Err(anyhow!("No items are found"));
    };
    let items_display = display_pairs(&items);

    let items_display_accumlated = items_display
        .iter()
        .fold("".to_owned(), |acc, i| acc + i + "\n");

    let echo = Command::new("echo")
        .arg(&items_display_accumlated)
        .stdout(Stdio::piped())
        .spawn()?;
    let menu = Command::new("sh")
        .arg("-c").arg(menu)
        .stdin(Stdio::from(echo.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .context("Cannot run menu. Your menu application might not be installed or you might have messed up your flags.")?;
    let selection =
        String::from_utf8(menu.wait_with_output()?.stdout).context("Output is not utf8")?;

    // Indexmap probably be better here but I am too lazy to figure it out
    // Also we remove the last character (it's a \n character)
    let selected_item_index_option = items_display
        .iter()
        .position(|i| i == &selection[..selection.len() - 1])
        ;
    // Sometimes selected item is not in the list. This makes sure that is passed as None
    let selected_item = match selected_item_index_option {
        Some(x) => {
            let extracted_item = items.get(x).unwrap();
            (Some(extracted_item.0), (Some(extracted_item.1.0), extracted_item.1.1))},
        None => (None, (None, selection.as_str())),
    };
    run_command(selected_item, runner, config)
}

/// run() but in the terminal using dialoguer select
pub fn terminal_run(
    collections_input: &[String],
    runner: Option<String>,
    config: &Config,
    _assume_yes: bool,
    selective: bool,
) -> Result<()> {
    // In the format of collection, keypair
    let items: Vec<(&str, (&str, &str))> = config
        .filter_collections(collections_input, selective)
        .iter()
        .flat_map(|i| {
            i.1.items
                .iter()
                .map(|j| (*i.0, (j.0.as_str(), j.1.as_str())))
        })
        .collect();
    if items.len() == 0 {
        return Err(anyhow!("No items are found"));
    }

    let selection = Select::new()
        .with_prompt("What do you choose? (arrow or vi keys)")
        .items(display_pairs(&items))
        .interact()
        .context("Cannot observe user input")?;
    let selected_item = items[selection];
    run_command(
        (
            Some(selected_item.0),
            (selected_item.1.0, selected_item.1.1),
        ),
        runner,
        config,
    )
}

/// Takes a vector pairs tupled with collection and convert them into a displayable format
pub fn display_pairs(pairs: &[(&str, (&str, &str))]) -> Vec<String> {
    // Check if values have duplicates
    let mut has_duplicate = IndexMap::new();
    for i in pairs {
        let _ = *has_duplicate
            .entry(i.1.0)
            .and_modify(|e| *e = true)
            .or_insert(false);
    }

    // Label each pair with a string
    pairs
        .iter()
        .map(|i| {
            // (There shouldn't be any default values)
            if *has_duplicate.entry(i.1.0).or_default() {
                format!("{} (from collection \"{}\")", i.1.0, i.0)
            } else {
                i.1.0.to_owned()
            }
        })
        .collect()
}

// T is a place holder, it doesn't actually matter
/// Run command with runner using data from to run function which extracts from config
pub fn run_command<T>(
    to_run: (Option<&str>, (T, &str)),
    runner: Option<String>,
    config: &Config,
) -> Result<()> {
    // 1. Check runner input
    // 2. Check default collection runner
    // 3. Check default config runner
    let command_string: &str = if runner.is_some() {
        &runner.unwrap()
    } else {
        match to_run.0 {
            Some(r) => match config.collections.get(r).unwrap().default_runner.as_ref() {
                Some(x) => x.as_str(),
                None => config.default_runner.as_str(),
            },
            None => config.default_runner.as_str(),
        }
    };
    let _ = Command::new("sh")
        .arg("-c")
        .arg(command_string)
        .arg("qtai")
        .arg(to_run.1.1)
        .status()
        .context("Cannot run the command.")?;
    Ok(())
}
