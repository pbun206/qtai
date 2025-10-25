use crate::collections::*;
use anyhow::*;
use colored::*;
use dialoguer::Select;
use indexmap::*;
use serde::*;

/// Configuration
#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Config {
    pub default_runner: String,
    pub default_menu: String,
    pub collections: IndexMap<String, Collection>,
}

impl Config {
    pub fn template() -> Self {
        Self {
            default_runner: "".to_string(),
            default_menu: "dmenu".to_string(),
            collections: IndexMap::default(),
        }
    }

    /// This method filters collections based on query
    pub fn filter_collections(
        self: &Self,
        collections_input: &[String],
        selective: bool,
    ) -> IndexMap<&str, &Collection> {
        // If there are no input, we input all of them
        if collections_input.len() == 0 {
            self.collections
                .iter()
                .map(|i| (i.0.as_str(), i.1))
                .collect()
        } else {
            if selective {
                self.collections
                    .iter()
                    .filter(|i| {
                        collections_input
                            .iter()
                            .any(|j| i.0.to_lowercase() == j.to_lowercase())
                    })
                    .map(|i| (i.0.as_str(), i.1))
                    .collect()
            } else {
                self.collections
                    .iter()
                    .filter(|i| {
                        collections_input
                            .iter()
                            .any(|j| i.0.to_lowercase().contains(&j.to_lowercase()))
                    })
                    .map(|i| (i.0.as_str(), i.1))
                    .collect()
            }
        }
    }

    /// This method filters collections based on arguments and then prints them.
    pub fn list_collections(self: &Self, collections_input: &[String], selective: bool) -> Result<()> {
        let collections: IndexMap<&str, &Collection> = self.filter_collections(collections_input, selective);
        if collections.len() == 0 {
            return Err(anyhow!("No collections are found"));
        }

        for i in collections {
            println!("{}", i.0.bold());
            if i.1.items.len() == 0 {
                println!("This collection is empty.");
            }
            for j in &i.1.items {
                println!("\"{}\": \"{}\"", j.0, j.1);
            }
            println!();
        }
        Ok(())
    }

    /// This method filters collections based on (&String) query and outputs a vector of potential candidates
    pub fn query_collections(self: &Self, query: &str) -> Vec<(&str, &Collection)> {
        self.collections
            .iter()
            .filter(|i| i.0.to_lowercase().contains(&query.to_lowercase()))
            .map(|i| (i.0.as_str(), i.1))
            .collect()
    }

    /// This method searches collections based on query and enables the user to select which one to use, returning the selection.
    pub fn select_collections(self: &Self, query: &str) -> Result<(&str, &Collection)> {
        if self.collections.len() == 0 {
            return Err(anyhow!("No collections are found!"));
        }
        let filtered_collections = self.query_collections(query);
        if filtered_collections.len() == 0 {
            Err(anyhow!("Cannot find any collections with that query."))
        } else if filtered_collections.len() == 1 {
            Ok(*filtered_collections.first().unwrap())
        } else {
            let collections_names = filtered_collections.iter().map(|x| x.0);
            println!("Multiple collections had been found.");
            let selection = Select::new()
                .with_prompt("What do you choose? (arrow or vi keys)")
                .items(collections_names)
                .interact()
                .context("Cannot observe user input")?;
            Ok(*filtered_collections
                .get(selection)
                .context("Index out of bounds")?)
        }
    }

    pub fn query_items(self: &Self, query: &str) -> Vec<(&str, Vec<(&str, &str)>)> {
        self.collections
            .iter()
            .map(|i| (i.0.as_str(), i.1.query_items(query)))
            .filter(|i| i.1.len() != 0)
            .collect()
    }

    /// This method searches items based on query and prints results.
    pub fn search_items(self: &Self, query: &str) -> Result<()> {
        let results = self.query_items(query);
        let mut count = 0;
        for i in results {
            count += 1;
            println!("{}", i.0.bold());
            for j in &i.1 {
                println!("\"{}\": \"{}\"", j.0, j.1);
            }
            println!();
        }
        if count == 0 {
            Err(anyhow!("Cannot find any results with query."))
        } else {
            Ok(())
        }
    }

    /// A function that queries a set of items. allowing the user to make the final choice, and outputs that collection name and item name.
    pub fn select_items(self: &Self, query: &str) -> Result<(&str, (&str, &str))> {
        let results: Vec<(&str, (&str, &str))> = self
            .query_items(query)
            .iter()
            .flat_map(|i| i.1.iter().map(|j| (i.0, *j)))
            .collect();
        if results.len() == 0 {
            Err(anyhow!("Cannot find any items with that query."))
        } else if results.len() == 1 {
            Ok(*results.first().unwrap())
        } else {
            let results_display: Vec<String> = results
                .iter()
                .map(|i| {
                    format!(
                        "\"{}\": \"{}\" from collection \"{}\"",
                        i.1.0.bold(),
                        i.1.1.bold(),
                        i.0.bold()
                    )
                })
                .collect();
            let selection = Select::new()
                .with_prompt("What do you choose? (arrow or vi keys)")
                .items(results_display)
                .interact()
                .context("Cannot observe user input")?;
            Ok(*results.get(selection).context("Index out of bounds")?)
        }
    }
}
