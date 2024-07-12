//! Delta Sharing Protocol message types

use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use serde_json::Map;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub share_credentials_version: i32,
    pub endpoint: String,
    pub bearer_token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Share {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub share: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Table {
    pub name: String,
    pub share: String,
    pub schema: String,
}

impl Table {
    pub fn fully_qualified_name(&self) -> String {
        format!("{}.{}.{}", self.share, self.schema, self.name)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaProtocol {
    pub min_reader_version: i32,
    pub min_writer_version: i32,
    pub reader_features: Vec<String>,
    pub writer_features: Vec<String>
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Protocol {
    Legacy { 
        #[serde(rename = "minReaderVersion")]
        min_reader_version: i32 
    },
    Delta { 
        #[serde(rename = "deltaProtocol")]
        delta_protocol: DeltaProtocol 
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Format {
    pub provider: String,
    pub options: Option<Map<String, Value>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyMetadata {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub format: Format,
    pub schema_string: String,
    pub configuration: Map<String, Value>,
    pub partition_columns: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaMetadata {
    pub size: usize,
    pub num_files: usize,
    pub delta_metadata: LegacyMetadata
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Metadata {
    Legacy (LegacyMetadata),
    Delta (DeltaMetadata)
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct TableMetadata {
    pub protocol: Protocol,
    pub metadata: Metadata,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: String,
    pub url: String,
    pub partition_values: Map<String, Value>,
    pub size: i32,
    pub stats: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct TableFiles {
    pub metadata: TableMetadata,
    pub files: Vec<File>,
}

pub struct FilesRequest {
    pub predicate_hints: Option<Vec<String>>,
    pub limit_hint: Option<i32>,
    pub version: Option<i32>,
}