//! ContentItem model - universal content wrapper.
//!
//! This is the primary model for representing any type of content in the Lens
//! ecosystem. It supports extensible metadata through standard schemas (Dublin Core,
//! Schema.org, DataCite, etc.) and arbitrary custom fields.

use super::content_types::{ContentType, Creator, License, Resource};
use super::metadata::MetadataContainer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal content item - base for all content types.
///
/// Supports any content type with extensible metadata through:
/// - Standard metadata schemas (Dublin Core, Schema.org, DataCite, etc.)
/// - Type-specific fields for specialized content
/// - Custom fields for arbitrary extensions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentItem {
    /// Unique identifier (hash/CID)
    pub id: String,

    /// Content type discriminator
    pub content_type: ContentType,

    /// Title/name
    pub title: String,

    /// Description/synopsis
    pub description: Option<String>,

    /// Creators/contributors
    #[serde(default)]
    pub creators: Vec<Creator>,

    /// Tags for categorization and search
    #[serde(default)]
    pub tags: Vec<String>,

    /// Primary language (ISO 639 code)
    pub language: Option<String>,

    /// License information
    pub license: Option<License>,

    /// Resources (files, thumbnails, etc.)
    #[serde(default)]
    pub resources: Vec<Resource>,

    /// Creation timestamp (ISO 8601)
    pub created_at: Option<String>,

    /// Last modified timestamp (ISO 8601)
    pub updated_at: Option<String>,

    /// Publication/release date (ISO 8601)
    pub published_at: Option<String>,

    /// Standard metadata (Dublin Core, Schema.org, etc.)
    #[serde(default)]
    pub metadata: MetadataContainer,

    /// Type-specific fields as JSON.
    /// Allows each content type to have unique fields.
    #[serde(default)]
    pub type_specific: serde_json::Value,

    /// Custom/arbitrary fields that don't fit elsewhere.
    /// Allows complete extensibility without breaking schema.
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,

    /// Schema version for this content item.
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

impl ContentItem {
    /// Create a new minimal content item.
    pub fn new(id: String, content_type: ContentType, title: String) -> Self {
        Self {
            id,
            content_type,
            title,
            description: None,
            creators: Vec::new(),
            tags: Vec::new(),
            language: None,
            license: None,
            resources: Vec::new(),
            created_at: None,
            updated_at: None,
            published_at: None,
            metadata: MetadataContainer::default(),
            type_specific: serde_json::Value::Null,
            custom: HashMap::new(),
            schema_version: default_schema_version(),
        }
    }

    /// Builder: Add description.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Builder: Add creator.
    pub fn with_creator(mut self, creator: Creator) -> Self {
        self.creators.push(creator);
        self
    }

    /// Builder: Add tag.
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Builder: Set tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder: Add resource.
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }

    /// Builder: Set license.
    pub fn with_license(mut self, license: License) -> Self {
        self.license = Some(license);
        self
    }

    /// Builder: Set language.
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }

    /// Builder: Set type-specific data.
    pub fn with_type_specific(mut self, data: serde_json::Value) -> Self {
        self.type_specific = data;
        self
    }

    /// Builder: Add custom field.
    pub fn with_custom_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }

    /// Builder: Set creation timestamp.
    pub fn with_created_at(mut self, timestamp: String) -> Self {
        self.created_at = Some(timestamp);
        self
    }

    /// Builder: Set publication date.
    pub fn with_published_at(mut self, date: String) -> Self {
        self.published_at = Some(date);
        self
    }

    /// Generate ID from content hash.
    pub fn generate_id(content: &[u8]) -> String {
        let hash = blake3::hash(content);
        hex::encode(hash.as_bytes())
    }

    /// DHT key prefix for content items.
    pub const DHT_PREFIX: &'static str = "content";

    /// Get the DHT key for this content item.
    pub fn dht_key(&self) -> citadel_dht::DhtKey {
        citadel_dht::hash_prefixed_key(Self::DHT_PREFIX, &self.id)
    }

    /// Get a thumbnail resource if present.
    pub fn thumbnail(&self) -> Option<&Resource> {
        self.resources
            .iter()
            .find(|r| r.purpose.as_deref() == Some("thumbnail"))
    }

    /// Get the primary content resource if present.
    pub fn primary_content(&self) -> Option<&Resource> {
        self.resources.iter().find(|r| {
            r.purpose.as_deref() == Some("primary") || r.purpose.as_deref() == Some("master")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::content_types::CreatorRole;

    #[test]
    fn new_content_item() {
        let item = ContentItem::new(
            "test123".to_string(),
            ContentType::Movie,
            "Test Movie".to_string(),
        );

        assert_eq!(item.id, "test123");
        assert_eq!(item.content_type, ContentType::Movie);
        assert_eq!(item.title, "Test Movie");
        assert_eq!(item.schema_version, "1.0.0");
    }

    #[test]
    fn content_item_builder() {
        let item = ContentItem::new(
            "test456".to_string(),
            ContentType::MusicAlbum,
            "Test Album".to_string(),
        )
        .with_description("A test music album".to_string())
        .with_creator(Creator::new(
            "Test Artist".to_string(),
            CreatorRole::Musician,
        ))
        .with_tag("rock".to_string())
        .with_tag("2024".to_string())
        .with_license(License::cc_by_4())
        .with_custom_field("custom_rating".to_string(), serde_json::json!(4.5));

        assert_eq!(item.description, Some("A test music album".to_string()));
        assert_eq!(item.creators.len(), 1);
        assert_eq!(item.tags.len(), 2);
        assert!(item.license.is_some());
        assert_eq!(
            item.custom.get("custom_rating"),
            Some(&serde_json::json!(4.5))
        );
    }

    #[test]
    fn content_item_serialization() {
        let item = ContentItem::new(
            "test789".to_string(),
            ContentType::ScientificPaper,
            "Research Paper".to_string(),
        )
        .with_description("Important research".to_string());

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ContentItem = serde_json::from_str(&json).unwrap();

        assert_eq!(item, deserialized);
    }

    #[test]
    fn diverse_content_types() {
        let types_to_test = vec![
            (ContentType::Movie, "Inception"),
            (ContentType::TvSeries, "Breaking Bad"),
            (ContentType::MusicAlbum, "Dark Side of the Moon"),
            (ContentType::Podcast, "Serial"),
            (ContentType::Book, "1984"),
            (ContentType::ScientificPaper, "On the Origin of Species"),
            (ContentType::Course, "Introduction to Rust"),
            (ContentType::Dataset, "Climate Data 2024"),
            (ContentType::AiModel, "GPT-4"),
            (ContentType::ContainerImage, "nginx:latest"),
            (ContentType::Photo, "Sunset.jpg"),
            (ContentType::MuseumArtifact, "Ancient Vase"),
            (ContentType::Backup, "System Backup 2024-01-01"),
            (ContentType::Custom("NFT".to_string()), "Bored Ape #1234"),
        ];

        for (content_type, title) in types_to_test {
            let item = ContentItem::new(
                format!("id_{}", title.replace(' ', "_")),
                content_type.clone(),
                title.to_string(),
            );

            let json = serde_json::to_string(&item).unwrap();
            let deserialized: ContentItem = serde_json::from_str(&json).unwrap();
            assert_eq!(item.content_type, deserialized.content_type);
            assert_eq!(item.title, deserialized.title);
        }
    }

    #[test]
    fn generate_id_deterministic() {
        let id1 = ContentItem::generate_id(b"test content");
        let id2 = ContentItem::generate_id(b"test content");
        assert_eq!(id1, id2);
    }

    #[test]
    fn resource_helpers() {
        let item = ContentItem::new("test".to_string(), ContentType::Movie, "Movie".to_string())
            .with_resource(Resource::thumbnail("QmThumb".to_string()))
            .with_resource(Resource {
                id: "QmMain".to_string(),
                mime_type: Some("video/mp4".to_string()),
                size: Some(1_000_000_000),
                checksum: None,
                purpose: Some("primary".to_string()),
            });

        assert!(item.thumbnail().is_some());
        assert_eq!(item.thumbnail().unwrap().id, "QmThumb");
        assert!(item.primary_content().is_some());
        assert_eq!(item.primary_content().unwrap().id, "QmMain");
    }
}
