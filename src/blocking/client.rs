use crate::protocol::*;
use crate::reader::*;
use crate::utils::*;
use parquet::data_type::AsBytes;
use polars::prelude::{LazyFrame, Result as PolarResult};
use reqwest::{header, header::HeaderValue};
use serde_json::{Map, Number, Value};
use std::collections::HashMap;
use std::env;
use std::{fs, io, path::Path, path::PathBuf};
use url::Url;

const METADATA_FILE: &str = "metadata.json";

pub struct Client {
    http_client: reqwest::blocking::Client,
    base_url: Url,
    pub data_root: String,
    cache: HashMap<String, FileCache>,
}

impl Client {
    pub fn new(
        provider_config: ProviderConfig,
        data_root: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        if provider_config.share_credentials_version > CREDENTIALS_VERSION {
            panic!("'share_credentials_version' in the provider configuration is {}, which is newer than the \
                    version {} supported by the current release. Please upgrade to a newer release.", 
                    provider_config.share_credentials_version,
                    CREDENTIALS_VERSION);
        }
        let cache: HashMap<String, FileCache> = HashMap::new();
        Ok(Self {
            http_client: Self::get_client(&provider_config)?,
            base_url: Self::build_base_url(&provider_config.endpoint)?,
            data_root: data_root.unwrap_or(
                env::temp_dir()
                    .as_path()
                    .join("delta_sharing")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            cache: cache,
        })
    }

    fn get_client(config: &ProviderConfig) -> Result<reqwest::blocking::Client, reqwest::Error> {
        let rust_version: &str = &format!("{}", rustc_version_runtime::version());
        let user_agent: &str = &format!("Delta-Sharing-Rust/{VERSION} Rust/{rust_version}");
        let bearer_token = &format!("Bearer {}", config.bearer_token);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(bearer_token).unwrap(),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );
        reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
    }

    fn build_base_url(endpoint: &String) -> Result<Url, url::ParseError> {
        let mut root_path = endpoint.trim_end_matches('/').to_string();
        root_path.push('/');
        Url::parse(&root_path)
    }

    fn get(&self, target: &str) -> Result<String, reqwest::Error> {
        let url = self.base_url.join(target).unwrap();
        debug!("--> HTTP GET to: {}", &url);
        let resp = self.http_client.get(url.as_str()).send()?;
        let resp_text = resp.text()?;
        debug!("--> Reponse body: {}", &resp_text);
        return Ok(resp_text);
    }

    fn head(&self, target: &str, key: &str) -> Option<HeaderValue> {
        let url = self.base_url.join(target).unwrap();
        debug!("HTTP HEAD to: {}", &url);
        let resp = self
            .http_client
            .head(url.as_str())
            .send()
            .expect("Invalid request");
        let version = resp.headers().get(key);
        match version {
            Some(h) => Some(h.clone()),
            None => None,
        }
    }

    fn post(&self, target: &str, json: &Map<String, Value>) -> Result<String, reqwest::Error> {
        let url = self.base_url.join(target).unwrap();
        debug!("--> HTTP POST to: {}", &url);
        let resp = self.http_client.post(url.as_str()).json(json).send()?;
        let resp_text = resp.text()?;
        debug!("--> Reponse body: {}", &resp_text);
        return Ok(resp_text);
    }

    fn download(&self, url: String, dest_path: &Path) {
        debug!("--> Download {} to {}", &url, dest_path.display());
        let resp = reqwest::blocking::get(url).unwrap();
        let mut out = fs::File::create(dest_path).expect("Failed to create an output file");
        let content = resp.bytes().unwrap();
        io::copy(&mut content.as_bytes(), &mut out)
            .expect("Failed to save the content to output file");
        // Ok(())
    }

    pub fn list_shares(&self) -> Result<Vec<Share>, anyhow::Error> {
        let shares = self.get("shares")?;
        let parsed: ShareResponse = serde_json::from_str(&shares).expect("Invalid response");
        return Ok(parsed.items.clone());
    }

    pub fn list_schemas(&self, share: &Share) -> Result<Vec<Schema>, anyhow::Error> {
        let schemas = self.get(&format!("shares/{}/schemas", share.name))?;
        let parsed: SchemaResponse = serde_json::from_str(&schemas).expect("Invalid response");
        return Ok(parsed.items.clone());
    }

    pub fn list_tables(&self, schema: &Schema) -> Result<Vec<Table>, anyhow::Error> {
        let tables = self.get(&format!(
            "shares/{}/schemas/{}/tables",
            schema.share, schema.name
        ))?;
        let parsed: TableResponse = serde_json::from_str(&tables).expect("Invalid response");
        return Ok(parsed.items.clone());
    }

    pub fn list_all_tables(&self, share: &Share) -> Result<Vec<Table>, anyhow::Error> {
        let tables = self.get(&format!("shares/{}/all-tables", share.name))?;
        let parsed: TableResponse = serde_json::from_str(&tables).expect("Invalid response");
        return Ok(parsed.items.clone());
    }

    pub fn get_table_metadata(&self, table: &Table) -> Result<TableMetadata, anyhow::Error> {
        let meta = self.get(&format!(
            "shares/{}/schemas/{}/tables/{}/metadata",
            table.share, table.schema, table.name
        ))?;
        let mut meta_lines = meta.lines();
        let protocol: ProtocolResponse =
            serde_json::from_str(meta_lines.next().expect("Invalid response"))
                .expect("Invalid protocol");
        let metadata: MetadataResponse =
            serde_json::from_str(meta_lines.next().expect("Invalid response"))
                .expect("Invalid metadata");
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
            Some(v) => v
                .to_str()
                .expect("Invalid version number")
                .parse::<i32>()
                .expect("Invalid version number"),
            None => -1,
        }
    }

    pub fn list_table_files(
        &self,
        table: &Table,
        predicate_hints: Option<Vec<String>>,
        limit_hint: Option<i32>,
        version: Option<i32>,
    ) -> Result<TableFiles, anyhow::Error> {
        let mut map = Map::new();
        if predicate_hints.is_some() {
            map.insert(
                "predicateHints".to_string(),
                Value::Array(
                    predicate_hints
                        .unwrap()
                        .iter()
                        .map(|s| Value::String(s.to_string()))
                        .collect::<Vec<_>>(),
                ),
            );
        }
        if limit_hint.is_some() {
            map.insert(
                "limitHint".to_string(),
                Value::Number(Number::from(limit_hint.unwrap())),
            );
        }
        if version.is_some() {
            map.insert(
                "version".to_string(),
                Value::Number(Number::from(version.unwrap())),
            );
        }
        let response = self.post(
            &format!(
                "shares/{}/schemas/{}/tables/{}/query",
                table.share, table.schema, table.name
            ),
            &map,
        )?;
        let mut lines = response.lines();
        let protocol: ProtocolResponse =
            serde_json::from_str(lines.next().expect("Invalid response"))
                .expect("Invalid protocol");
        let metadata: MetadataResponse =
            serde_json::from_str(lines.next().expect("Invalid response"))
                .expect("Invalid metadata");
        let mut files: Vec<File> = Vec::new();
        for l in lines {
            let file: FileResponse = serde_json::from_str(l).expect("Invalid file info");
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

    fn download_files(&self, table_path: &PathBuf, table_files: &TableFiles) -> Vec<PathBuf> {
        if Path::exists(&table_path) {
            fs::remove_dir_all(&table_path).unwrap();
        }
        fs::create_dir_all(&table_path).unwrap();
        let mut file_paths: Vec<PathBuf> = Vec::new();
        for file in table_files.files.clone() {
            let dst_path = &table_path.join(format!("{}.snappy.parquet", &file.id));
            self.download(file.url, &dst_path);
            file_paths.push(dst_path.clone());
        }
        file_paths.clone()
    }

    fn load_cached(&self, table_path: &PathBuf, table_files: &TableFiles) -> Option<Vec<PathBuf>> {
        // Check if the files exist, load and compare the files.
        let metadata_path = &table_path.join(METADATA_FILE);
        if Path::exists(&metadata_path) {
            let metadata_str = &fs::read_to_string(&metadata_path).unwrap();
            let metadata: TableMetadata = serde_json::from_str(&metadata_str).expect(&format!(
                "Invalid configuration in {}",
                metadata_path.display()
            ));
            let mut download = metadata != table_files.metadata;

            if !download {
                let mut file_paths: Vec<PathBuf> = Vec::new();
                for file in &table_files.files {
                    let file_path = &table_path.join(format!("{}.snappy.parquet", &file.id));
                    if !Path::exists(&file_path) {
                        // File is missing, invalidate cache
                        download = true;
                        fs::remove_dir(&table_path).unwrap();
                        break;
                    }
                    file_paths.push(file_path.clone());
                }
                if !download {
                    return Some(file_paths.clone());
                }
            }
        }
        None
    }

    pub fn get_files(&mut self, table: &Table) -> Result<Vec<PathBuf>, anyhow::Error> {
        let key = table.fully_qualified_name();
        let mut download = true;
        let table_path = Path::new(&self.data_root).join(table.fully_qualified_name());
        let table_files = self.list_table_files(table, None, None, None).unwrap();
        if let Some(cached) = self.cache.get(&key) {
            download = cached.table_files.metadata != table_files.metadata;
        } else if let Some(cached) = self.load_cached(&table_path, &table_files) {
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
            let paths = self.download_files(&table_path, &table_files);
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
        Ok(self.cache.get(&key).unwrap().file_paths.clone())
    }

    pub fn get_dataframe(&mut self, table: &Table) -> PolarResult<LazyFrame> {
        self.get_files(&table)?;
        let table_path = Path::new(&self.data_root).join(table.fully_qualified_name());
        load_parquet_files_as_dataframe(&table_path)
    }
}
