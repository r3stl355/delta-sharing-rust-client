pub const VERSION: &str = "1";
pub const CREDENTIALS_VERSION: i32 = 1;

use crate::protocol::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct ShareResponse {
    pub items: Vec<Share>,
}

#[derive(Deserialize, Debug)]
pub struct SchemaResponse {
    pub items: Vec<Schema>,
}

#[derive(Deserialize, Debug)]
pub struct TableResponse {
    pub items: Vec<Table>,
}

#[derive(Deserialize)]
pub struct ProtocolResponse {
    pub protocol: Protocol,
}

#[derive(Deserialize)]
pub struct MetadataResponse {
    #[serde(rename(deserialize = "metaData"))]
    pub metadata: Metadata,
}

#[derive(Deserialize)]
pub struct FileResponse {
    pub file: File,
}

#[derive(Deserialize, PartialEq, Serialize)]
pub struct FileCache {
    pub table_files: TableFiles,
    pub file_paths: Vec<PathBuf>,
}
