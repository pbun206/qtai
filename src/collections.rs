use indexmap::*;
use serde::*;

///Collection stores a profile which has a default runner
/// and collection of items to pair with the runner
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct Collection {
    pub default_runner: Option<String>,
    #[serde(flatten)]
    pub items: IndexMap<String, String>,
}

impl Collection {
    /// Outputs a vector of items which matches with query, ignorant of case
    pub fn query_items(self: &Self, query: &str) -> Vec<(&str, &str)> {
        self.items
            .iter()
            .filter(|i| {
                i.0.to_lowercase().contains(&query.to_lowercase())
                    || i.1.to_lowercase().contains(&query.to_lowercase())
            })
            .map(|i| (i.0.as_str(), i.1.as_str()))
            .collect()
    }

    pub fn template() -> Self {
        let mut template_items = IndexMap::new();
        template_items.insert("example key".to_string(), "example value".to_string());
        Self {
            default_runner: None,
            items: template_items,
        }
    }
}
