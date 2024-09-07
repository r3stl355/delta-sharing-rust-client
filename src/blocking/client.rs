use crate::protocol::*;
use crate::reader::*;
use crate::utils::*;
use parquet::data_type::AsBytes;
use polars::prelude::LazyFrame;
use reqwest::{header, header::HeaderValue};
use serde_json::{Map, Number, Value};
use std::collections::HashMap;
use std::env;
use std::{fs, io, path::Path, path::PathBuf};
use url::Url;

const METADATA_FILE: &str = "metadata.json";

/// A blocking Client for working with Data Sharing
pub struct Client {
    http_client: reqwest::blocking::Client,
    base_url: Url,
    /// Local directory path to store the downloaded cached files
    pub data_root: String,
    cache: HashMap<String, FileCache>,
}

impl Client {
    /// Constructs a new blocking Client
    /// # Arguments
    ///
    /// * `provider_config` - Delta Sharing Provider Configuration of type [ProviderConfig]
    /// * `data_root` - An optional local directory path for caching. Temp location is used if None is given
    pub fn new(
        provider_config: ProviderConfig,
        data_root: Option<String>,
        capabilities: Option<HashMap<String, String>>,
    ) -> Result<Self, anyhow::Error> {
        if provider_config.share_credentials_version > CREDENTIALS_VERSION {
            return Err(anyhow::anyhow!("'share_credentials_version' in the provider configuration is {}, which is newer than the \
                    version {} supported by the current release. Please upgrade to a newer release.", 
                    provider_config.share_credentials_version,
                    CREDENTIALS_VERSION));
        }
        let cache: HashMap<String, FileCache> = HashMap::new();
        Ok(Self {
            http_client: Self::get_client(&provider_config, capabilities.unwrap_or_default())?,
            base_url: Self::build_base_url(&provider_config.endpoint)?,
            data_root: data_root.unwrap_or(
                env::temp_dir()
                    .as_path()
                    .join("delta_sharing")
                    .to_str()
                    .ok_or(anyhow::anyhow!("Error selecting data root folder"))?
                    .to_string(),
            ),
            cache: cache,
        })
    }

    fn get_client(
        config: &ProviderConfig,
        capabilities: HashMap<String, String>,
    ) -> Result<reqwest::blocking::Client, anyhow::Error> {
        let rust_version: &str = &format!("{}", rustc_version_runtime::version());
        let user_agent: &str = &format!("Delta-Sharing-Rust/{VERSION} Rust/{rust_version}");
        let bearer_token = &format!("Bearer {}", config.bearer_token);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(bearer_token)
                .map_err(|e| anyhow::anyhow!("Error setting authorization header:{e}"))?,
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent)
                .map_err(|e| anyhow::anyhow!("Error setting user agent header:{e}"))?,
        );
        headers.insert(
            header::HeaderName::from_static("delta-sharing-capabilities"),
            header::HeaderValue::from_str(
                &capabilities
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<String>>()
                    .join(";"),
            )
            .map_err(|e| anyhow::anyhow!("Error setting delta-sharing-capabilities header:{e}"))?,
        );
        reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| anyhow::anyhow!("Error building Http client: {e}"))
    }

    fn build_base_url(endpoint: &String) -> Result<Url, url::ParseError> {
        let mut root_path = endpoint.trim_end_matches('/').to_string();
        root_path.push('/');
        Url::parse(&root_path)
    }

    fn get(&self, target: &str) -> Result<String, anyhow::Error> {
        let url = self
            .base_url
            .join(target)
            .map_err(|e| anyhow::anyhow!("Error creating GET url: {e}"))?;
        debug!("--> HTTP GET to: {}", &url);
        let resp = self.http_client.get(url.as_str()).send()?;
        let resp_text = resp.text()?;
        debug!("--> Reponse body: {}", &resp_text);
        return Ok(resp_text);
    }

    fn head(&self, target: &str, key: &str) -> Result<Option<HeaderValue>, anyhow::Error> {
        let url = self
            .base_url
            .join(target)
            .map_err(|e| anyhow::anyhow!("Error creating HEAD url: {e}"))?;
        debug!("HTTP HEAD to: {}", &url);
        let resp = self
            .http_client
            .head(url.as_str())
            .send()
            .map_err(|e| anyhow::anyhow!("Invalid HEAD request: {e}"))?;
        let version = resp.headers().get(key);
        match version {
            Some(h) => Ok(Some(h.clone())),
            None => Ok(None),
        }
    }

    fn post(&self, target: &str, json: &Map<String, Value>) -> Result<String, anyhow::Error> {
        let url = self
            .base_url
            .join(target)
            .map_err(|e| anyhow::anyhow!("Error creating POST url: {e}"))?;
        debug!("--> HTTP POST to: {}", &url);
        let resp = self.http_client.post(url.as_str()).json(json).send()?;
        let resp_text = resp.text()?;
        debug!("--> Reponse body: {}", &resp_text);
        return Ok(resp_text);
    }

    fn download(&self, url: String, dest_path: &Path) -> Result<u64, anyhow::Error> {
        debug!("--> Download {} to {}", &url, dest_path.display());
        let resp = reqwest::blocking::get(url)
            .map_err(|e| anyhow::anyhow!("Error creating POST url: {e}"))?;
        let mut out = fs::File::create(dest_path)
            .map_err(|e| anyhow::anyhow!("Failed to create an output file: {e}"))?;
        let content = resp
            .bytes()
            .map_err(|e| anyhow::anyhow!("Failed to read download bytes: {e}"))?;
        io::copy(&mut content.as_bytes(), &mut out)
            .map_err(|e| anyhow::anyhow!("Failed to save the content to output file: {e}"))
    }

    pub fn list_shares(&self) -> Result<Vec<Share>, anyhow::Error> {
        let shares = self.get("shares")?;
        let parsed: ShareResponse = serde_json::from_str(&shares)
            .map_err(|e| anyhow::anyhow!("Invalid list shares response: {e}"))?;
        return Ok(parsed.items.clone());
    }

    pub fn list_schemas(&self, share: &Share) -> Result<Vec<Schema>, anyhow::Error> {
        let schemas = self.get(&format!("shares/{}/schemas", share.name))?;
        let parsed: SchemaResponse = serde_json::from_str(&schemas)
            .map_err(|e| anyhow::anyhow!("Invalid list schemas response: {e}"))?;
        return Ok(parsed.items.clone());
    }

    pub fn list_tables(&self, schema: &Schema) -> Result<Vec<Table>, anyhow::Error> {
        let tables = self.get(&format!(
            "shares/{}/schemas/{}/tables",
            schema.share, schema.name
        ))?;
        let parsed: TableResponse = serde_json::from_str(&tables)
            .map_err(|e| anyhow::anyhow!("Invalid list tables response: {e}"))?;
        return Ok(parsed.items.clone());
    }

    pub fn list_all_tables(&self, share: &Share) -> Result<Vec<Table>, anyhow::Error> {
        let tables = self.get(&format!("shares/{}/all-tables", share.name))?;
        let parsed: TableResponse = serde_json::from_str(&tables)
            .map_err(|e| anyhow::anyhow!("Invalid list all tables response: {e}"))?;
        return Ok(parsed.items.clone());
    }

    pub fn get_table_metadata(&self, table: &Table) -> Result<TableMetadata, anyhow::Error> {
        let meta = self.get(&format!(
            "shares/{}/schemas/{}/tables/{}/metadata",
            table.share, table.schema, table.name
        ))?;
        let mut meta_lines = meta.lines();
        let protocol: ProtocolResponse = meta_lines
            .next()
            .map(|lines| {
                serde_json::from_str::<ProtocolResponse>(lines)
                    .map_err(|e| anyhow::anyhow!("Invalid protocol response - {lines}: {e}"))
            })
            .unwrap_or(Err(anyhow::anyhow!("Empty protocol response")))?;
        let metadata: MetadataResponse = meta_lines
            .next()
            .map(|lines| {
                serde_json::from_str::<MetadataResponse>(lines)
                    .map_err(|e| anyhow::anyhow!("Invalid metadata response - {lines}: {e}"))
            })
            .unwrap_or(Err(anyhow::anyhow!("Empty metadata response")))?;
        Ok(TableMetadata {
            protocol: protocol.protocol,
            metadata: metadata.metadata,
        })
    }

    pub fn get_table_version(&self, table: &Table) -> i32 {
        let version = self.head(
            &format!(
                "shares/{}/schemas/{}/tables/{}",
                table.share, table.schema, table.name
            ),
            "delta-table-version",
        );
        match version {
            Ok(Some(v)) => v
                .to_str()
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(-1),
            _ => -1,
        }
    }

    pub fn list_table_files(
        &self,
        table: &Table,
        request: Option<FilesRequest>,
    ) -> Result<TableFiles, anyhow::Error> {
        let mut map = Map::new();
        if let Some(predicate_hints) = request.as_ref().and_then(|r| r.predicate_hints.as_ref()) {
            map.insert(
                "predicateHints".to_string(),
                Value::Array(
                    predicate_hints
                        .iter()
                        .map(|s| Value::String(s.to_string()))
                        .collect::<Vec<_>>(),
                ),
            );
        }
        if let Some(limit_hint) = request.as_ref().and_then(|r| r.limit_hint) {
            map.insert(
                "limitHint".to_string(),
                Value::Number(Number::from(limit_hint)),
            );
        }
        if let Some(version) = request.as_ref().and_then(|r| r.version) {
            map.insert("version".to_string(), Value::Number(Number::from(version)));
        }
        let response = self.post(
            &format!(
                "shares/{}/schemas/{}/tables/{}/query",
                table.share, table.schema, table.name
            ),
            &map,
        )?;
        let mut lines = response.lines();
        let protocol: ProtocolResponse = lines
            .next()
            .map(|lines| {
                serde_json::from_str::<ProtocolResponse>(lines)
                    .map_err(|e| anyhow::anyhow!("Invalid protocol response - {lines}: {e}"))
            })
            .unwrap_or(Err(anyhow::anyhow!("Empty protocol response")))?;
        let metadata: MetadataResponse = lines
            .next()
            .map(|lines| {
                serde_json::from_str::<MetadataResponse>(lines)
                    .map_err(|e| anyhow::anyhow!("Invalid metadata response - {lines}: {e}"))
            })
            .unwrap_or(Err(anyhow::anyhow!("Empty metadata response")))?;
        let mut files: Vec<File> = Vec::new();
        for l in lines {
            let file: FileResponse =
                serde_json::from_str(l).map_err(|e| anyhow::anyhow!("Invalid file info: {e}"))?;
            files.push(file.file.clone());
        }
        Ok(TableFiles {
            metadata: TableMetadata {
                protocol: protocol.protocol,
                metadata: metadata.metadata,
            },
            files,
        })
    }

    fn download_files(
        &self,
        table_path: &PathBuf,
        table_files: &TableFiles,
    ) -> Result<Vec<PathBuf>, anyhow::Error> {
        if Path::exists(&table_path) {
            fs::remove_dir_all(&table_path)
                .map_err(|e| anyhow::anyhow!("Error cleaning table path: {e}"))?;
        }
        fs::create_dir_all(&table_path)
            .map_err(|e| anyhow::anyhow!("Error creating table path: {e}"))?;
        let mut file_paths: Vec<PathBuf> = Vec::new();
        let count = table_files.files.len();
        for (index, file) in table_files.files.clone().into_iter().enumerate() {
            match file {
                File::Parquet(ParquetFile { id, url, .. }) => {
                    let dst_path = &table_path.join(format!("{}.snappy.parquet", &id));
                    let bytes = self.download(url, &dst_path)?;
                    debug!(
                        "Downloaded {}/{} {} ({} bytes)",
                        index + 1,
                        count,
                        dst_path.display(),
                        bytes
                    );
                    file_paths.push(dst_path.clone());
                }
                File::Delta(delta_file) => {
                    if let Some(url) = delta_file.get_url() {
                        let dst_path =
                            &table_path.join(format!("{}.snappy.parquet", &delta_file.id));
                        let bytes = self.download(url, &dst_path)?;
                        debug!(
                            "Downloaded {}/{} {} ({} bytes)",
                            index + 1,
                            count,
                            dst_path.display(),
                            bytes
                        );
                        file_paths.push(dst_path.clone());
                    }
                }
            }
        }
        Ok(file_paths.clone())
    }

    fn load_cached(
        &self,
        table_path: &PathBuf,
        table_files: &TableFiles,
    ) -> Result<Option<Vec<PathBuf>>, anyhow::Error> {
        // Check if the files exist, load and compare the files.
        let metadata_path = &table_path.join(METADATA_FILE);
        if Path::exists(&metadata_path) {
            let metadata_str = &fs::read_to_string(&metadata_path).map_err(|e| {
                anyhow::anyhow!("Error reading file path {}: {}", metadata_path.display(), e)
            })?;
            let metadata: TableMetadata = serde_json::from_str(&metadata_str).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid configuration in {}: {}",
                    metadata_path.display(),
                    e
                )
            })?;
            let mut download = metadata != table_files.metadata;

            if !download {
                let mut file_paths: Vec<PathBuf> = Vec::new();
                for file in &table_files.files {
                    let file_id = match file {
                        File::Parquet(ParquetFile { id, .. }) => id,
                        File::Delta(DeltaFile { id, .. }) => id,
                    };
                    let file_path = &table_path.join(format!("{}.snappy.parquet", &file_id));
                    if !Path::exists(&file_path) {
                        // File is missing, invalidate cache
                        download = true;
                        fs::remove_dir_all(&table_path)
                            .map_err(|e| anyhow::anyhow!("Error invalidating cache: {e}"))?;
                        break;
                    }
                    file_paths.push(file_path.clone());
                }
                if !download {
                    return Ok(Some(file_paths.clone()));
                }
            }
        }
        Ok(None)
    }

    pub fn get_files(
        &mut self,
        table: &Table,
        request: Option<FilesRequest>,
    ) -> Result<Vec<PathBuf>, anyhow::Error> {
        let key = table.fully_qualified_name();
        let mut download = true;
        let table_path = Path::new(&self.data_root).join(table.fully_qualified_name());
        let table_files = self.list_table_files(table, request)?;
        if let Some(cached) = self.cache.get(&key) {
            download = cached.table_files.metadata != table_files.metadata;
        } else if let Some(cached) = self.load_cached(&table_path, &table_files)? {
            download = false;
            self.cache.insert(
                key.clone(),
                FileCache {
                    table_files: table_files.clone(),
                    file_paths: cached,
                },
            );
        }
        if download {
            info!("--> Downloading data files to {}", &table_path.display());
            let paths = self.download_files(&table_path, &table_files)?;
            serde_json::to_writer(
                &fs::File::create(&table_path.join(METADATA_FILE))?,
                &table_files.metadata,
            )?;
            self.cache.insert(
                key.clone(),
                FileCache {
                    table_files: table_files,
                    file_paths: paths,
                },
            );
        }
        Ok(self
            .cache
            .get(&key)
            .ok_or(anyhow::anyhow!("Error reading {key} from cache"))?
            .file_paths
            .clone())
    }

    pub fn get_dataframe(
        &mut self,
        table: &Table,
        request: Option<FilesRequest>,
    ) -> Result<LazyFrame, anyhow::Error> {
        self.get_files(&table, request)?;
        let table_path = Path::new(&self.data_root).join(table.fully_qualified_name());
        load_parquet_files_as_dataframe(&table_path)
            .map_err(|e| anyhow::anyhow!("Error loading parquet files: {e}"))
    }
}
