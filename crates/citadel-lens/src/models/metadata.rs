//! Standard metadata schemas support.
//!
//! Supports Dublin Core, Schema.org, DataCite, Darwin Core, PREMIS, and custom schemas.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard metadata schemas support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "standard", rename_all = "snake_case")]
pub enum StandardMetadata {
    /// Dublin Core metadata.
    /// https://www.dublincore.org/specifications/dublin-core/dcmi-terms/
    DublinCore {
        /// Title
        title: Option<String>,
        /// Creator
        creator: Option<Vec<String>>,
        /// Subject/keywords
        subject: Option<Vec<String>>,
        /// Description
        description: Option<String>,
        /// Publisher
        publisher: Option<String>,
        /// Contributor
        contributor: Option<Vec<String>>,
        /// Date
        date: Option<String>,
        /// Type
        #[serde(rename = "type")]
        dc_type: Option<String>,
        /// Format
        format: Option<String>,
        /// Identifier
        identifier: Option<String>,
        /// Source
        source: Option<String>,
        /// Language
        language: Option<String>,
        /// Relation
        relation: Option<String>,
        /// Coverage
        coverage: Option<String>,
        /// Rights
        rights: Option<String>,
    },

    /// Schema.org structured data.
    /// https://schema.org/
    SchemaOrg {
        /// @context
        context: String,
        /// @type
        #[serde(rename = "type")]
        schema_type: String,
        /// Properties as key-value pairs
        properties: HashMap<String, serde_json::Value>,
    },

    /// DataCite metadata schema (for research data).
    /// https://schema.datacite.org/
    DataCite {
        /// DOI
        doi: Option<String>,
        /// Creators
        creators: Vec<DataCiteCreator>,
        /// Titles
        titles: Vec<DataCiteTitle>,
        /// Publisher
        publisher: String,
        /// Publication year
        publication_year: u32,
        /// Resource type
        resource_type: DataCiteResourceType,
        /// Subjects
        subjects: Option<Vec<String>>,
        /// Additional properties
        #[serde(flatten)]
        additional: HashMap<String, serde_json::Value>,
    },

    /// Darwin Core (for biodiversity/specimens).
    /// https://dwc.tdwg.org/
    DarwinCore {
        /// Scientific name
        scientific_name: Option<String>,
        /// Kingdom
        kingdom: Option<String>,
        /// Phylum
        phylum: Option<String>,
        /// Class
        class: Option<String>,
        /// Order
        order: Option<String>,
        /// Family
        family: Option<String>,
        /// Genus
        genus: Option<String>,
        /// Additional DwC terms
        #[serde(flatten)]
        additional: HashMap<String, serde_json::Value>,
    },

    /// PREMIS (digital preservation).
    /// https://www.loc.gov/standards/premis/
    Premis {
        /// Object identifier
        object_id: String,
        /// Object category
        object_category: String,
        /// Preservation level
        preservation_level: Option<String>,
        /// Additional PREMIS data
        #[serde(flatten)]
        additional: HashMap<String, serde_json::Value>,
    },

    /// Custom/arbitrary metadata.
    /// Allows any metadata scheme not explicitly supported.
    Custom {
        /// Schema identifier (URL or name)
        schema: String,
        /// Metadata as arbitrary JSON
        data: serde_json::Value,
    },
}

/// DataCite creator information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataCiteCreator {
    pub name: String,
    pub name_type: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub name_identifiers: Option<Vec<NameIdentifier>>,
}

impl DataCiteCreator {
    /// Create a personal name creator.
    pub fn personal(given_name: &str, family_name: &str) -> Self {
        Self {
            name: format!("{}, {}", family_name, given_name),
            name_type: Some("Personal".to_string()),
            given_name: Some(given_name.to_string()),
            family_name: Some(family_name.to_string()),
            name_identifiers: None,
        }
    }

    /// Create an organizational creator.
    pub fn organizational(name: &str) -> Self {
        Self {
            name: name.to_string(),
            name_type: Some("Organizational".to_string()),
            given_name: None,
            family_name: None,
            name_identifiers: None,
        }
    }

    /// Add an ORCID identifier.
    pub fn with_orcid(mut self, orcid: &str) -> Self {
        let identifier = NameIdentifier {
            name_identifier: orcid.to_string(),
            name_identifier_scheme: "ORCID".to_string(),
            scheme_uri: Some("https://orcid.org".to_string()),
        };
        self.name_identifiers
            .get_or_insert_with(Vec::new)
            .push(identifier);
        self
    }
}

/// Name identifier (ORCID, ISNI, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameIdentifier {
    pub name_identifier: String,
    pub name_identifier_scheme: String,
    pub scheme_uri: Option<String>,
}

/// DataCite title with optional type and language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataCiteTitle {
    pub title: String,
    pub title_type: Option<String>,
    pub lang: Option<String>,
}

impl DataCiteTitle {
    /// Create a main title.
    pub fn main(title: &str) -> Self {
        Self {
            title: title.to_string(),
            title_type: None,
            lang: None,
        }
    }

    /// Create a subtitle.
    pub fn subtitle(title: &str) -> Self {
        Self {
            title: title.to_string(),
            title_type: Some("Subtitle".to_string()),
            lang: None,
        }
    }

    /// Create a translated title.
    pub fn translated(title: &str, lang: &str) -> Self {
        Self {
            title: title.to_string(),
            title_type: Some("TranslatedTitle".to_string()),
            lang: Some(lang.to_string()),
        }
    }
}

/// DataCite resource type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataCiteResourceType {
    pub resource_type_general: String,
    pub resource_type: Option<String>,
}

impl DataCiteResourceType {
    /// Create a dataset resource type.
    pub fn dataset(specific: Option<&str>) -> Self {
        Self {
            resource_type_general: "Dataset".to_string(),
            resource_type: specific.map(|s| s.to_string()),
        }
    }

    /// Create a software resource type.
    pub fn software(specific: Option<&str>) -> Self {
        Self {
            resource_type_general: "Software".to_string(),
            resource_type: specific.map(|s| s.to_string()),
        }
    }

    /// Create a text resource type.
    pub fn text(specific: Option<&str>) -> Self {
        Self {
            resource_type_general: "Text".to_string(),
            resource_type: specific.map(|s| s.to_string()),
        }
    }
}

/// Container for multiple metadata standards.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MetadataContainer {
    /// Multiple metadata standards can coexist
    #[serde(default)]
    pub standards: Vec<StandardMetadata>,
}

impl MetadataContainer {
    /// Create an empty metadata container.
    pub fn new() -> Self {
        Self {
            standards: Vec::new(),
        }
    }

    /// Add a metadata standard.
    pub fn add(&mut self, metadata: StandardMetadata) {
        self.standards.push(metadata);
    }

    /// Get Dublin Core metadata if present.
    pub fn get_dublin_core(&self) -> Option<&StandardMetadata> {
        self.standards
            .iter()
            .find(|m| matches!(m, StandardMetadata::DublinCore { .. }))
    }

    /// Get Schema.org metadata if present.
    pub fn get_schema_org(&self) -> Option<&StandardMetadata> {
        self.standards
            .iter()
            .find(|m| matches!(m, StandardMetadata::SchemaOrg { .. }))
    }

    /// Get DataCite metadata if present.
    pub fn get_datacite(&self) -> Option<&StandardMetadata> {
        self.standards
            .iter()
            .find(|m| matches!(m, StandardMetadata::DataCite { .. }))
    }

    /// Check if any metadata is present.
    pub fn is_empty(&self) -> bool {
        self.standards.is_empty()
    }

    /// Number of metadata standards present.
    pub fn len(&self) -> usize {
        self.standards.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dublin_core_metadata() {
        let dc = StandardMetadata::DublinCore {
            title: Some("Example Dataset".to_string()),
            creator: Some(vec!["Jane Doe".to_string()]),
            subject: Some(vec!["Science".to_string(), "Data".to_string()]),
            description: Some("An example dataset".to_string()),
            publisher: Some("Example Org".to_string()),
            contributor: None,
            date: Some("2024-01-01".to_string()),
            dc_type: Some("Dataset".to_string()),
            format: Some("application/json".to_string()),
            identifier: Some("doi:10.1234/example".to_string()),
            source: None,
            language: Some("en".to_string()),
            relation: None,
            coverage: None,
            rights: Some("CC-BY-4.0".to_string()),
        };

        let json = serde_json::to_string(&dc).unwrap();
        let deserialized: StandardMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(dc, deserialized);
    }

    #[test]
    fn custom_metadata() {
        let custom = StandardMetadata::Custom {
            schema: "https://example.com/my-schema".to_string(),
            data: serde_json::json!({
                "custom_field": "value",
                "another_field": 42
            }),
        };

        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: StandardMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(custom, deserialized);
    }

    #[test]
    fn metadata_container() {
        let mut container = MetadataContainer::new();
        assert!(container.is_empty());

        container.add(StandardMetadata::DublinCore {
            title: Some("Test".to_string()),
            creator: None,
            subject: None,
            description: None,
            publisher: None,
            contributor: None,
            date: None,
            dc_type: None,
            format: None,
            identifier: None,
            source: None,
            language: None,
            relation: None,
            coverage: None,
            rights: None,
        });

        assert!(!container.is_empty());
        assert_eq!(container.len(), 1);
        assert!(container.get_dublin_core().is_some());
        assert!(container.get_schema_org().is_none());
    }

    #[test]
    fn datacite_creator() {
        let creator = DataCiteCreator::personal("John", "Smith").with_orcid("0000-0001-2345-6789");

        assert_eq!(creator.name, "Smith, John");
        assert!(creator.name_identifiers.is_some());

        let json = serde_json::to_string(&creator).unwrap();
        let deserialized: DataCiteCreator = serde_json::from_str(&json).unwrap();
        assert_eq!(creator, deserialized);
    }

    #[test]
    fn datacite_metadata() {
        let datacite = StandardMetadata::DataCite {
            doi: Some("10.1234/example".to_string()),
            creators: vec![DataCiteCreator::personal("John", "Smith")],
            titles: vec![DataCiteTitle::main("Example Research Data")],
            publisher: "Example University".to_string(),
            publication_year: 2024,
            resource_type: DataCiteResourceType::dataset(Some("Experimental Data")),
            subjects: Some(vec!["Biology".to_string(), "Genetics".to_string()]),
            additional: HashMap::new(),
        };

        let json = serde_json::to_string(&datacite).unwrap();
        let deserialized: StandardMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(datacite, deserialized);
    }
}
