//! Category model for content organization.

use serde::{Deserialize, Serialize};

/// A category for organizing content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Parent category ID (for hierarchies)
    pub parent_id: Option<String>,

    /// Icon identifier
    pub icon: Option<String>,
}

impl Category {
    /// Create a new category.
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            parent_id: None,
            icon: None,
        }
    }

    /// DHT key prefix for categories.
    pub const DHT_PREFIX: &'static str = "category";

    /// Get the DHT key for this category.
    pub fn dht_key(&self) -> citadel_dht::DhtKey {
        citadel_dht::hash_prefixed_key(Self::DHT_PREFIX, &self.id)
    }

    /// Default categories for common content types.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::new("music".to_string(), "Music".to_string()),
            Self::new("movies".to_string(), "Movies".to_string()),
            Self::new("tv".to_string(), "TV Shows".to_string()),
            Self::new("games".to_string(), "Games".to_string()),
            Self::new("books".to_string(), "Books".to_string()),
            Self::new("software".to_string(), "Software".to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_category() {
        let cat = Category::new("music".to_string(), "Music".to_string());
        assert_eq!(cat.id, "music");
        assert_eq!(cat.name, "Music");
    }

    #[test]
    fn defaults_not_empty() {
        let defaults = Category::defaults();
        assert!(!defaults.is_empty());
        assert!(defaults.iter().any(|c| c.id == "music"));
    }
}
