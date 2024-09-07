mod common;

use common::create_mocked_test_app;
use delta_sharing::protocol::*;
use std::path::Path;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

// #[cfg(not(feature = "blocking"))]
#[tokio::test]
async fn list_shares() {
    let body =
        r#"{"items": [ { "name": "share_1","id": "1" }, { "name": "share_2", "id": "2" } ]}"#;
    let app = create_mocked_test_app(body, "/shares", method("GET")).await;
    let shares = app.client.list_shares().await.unwrap();

    assert_eq!(
        shares.len(),
        2,
        "Expected {} items, got {}",
        2,
        shares.len()
    );
}

#[tokio::test]
async fn list_schemas() {
    let share = Share {
        name: "share_1".to_string(),
    };
    let body = &format!(
        r#"{{ "items": [ {{ "name":"schema_1", "share": "{0}" }}, {{ "name": "schema_2", "share": "{0}" }} ] }}"#,
        share.name
    );
    let url = format!("/shares/{}/schemas", share.name);
    let app = create_mocked_test_app(body, &url, method("GET")).await;

    let schemas = app.client.list_schemas(&share).await.unwrap();

    assert_eq!(
        schemas.len(),
        2,
        "Expected {} items, got {}",
        2,
        schemas.len()
    );
}

#[tokio::test]
async fn list_tables() {
    let schema = Schema {
        name: "share_1".to_string(),
        share: "schema_1".to_string(),
    };
    let body = &format!(
        r#"{{ "items": [ {{ "name":"table_1", "share": "{0}", "schema": "{1}" }}, {{ "name": "table_2", "share": "{0}", "schema": "{1}" }} ] }}"#,
        schema.share, schema.name
    );
    let url = format!("shares/{}/schemas/{}/tables", schema.share, schema.name);
    let app = create_mocked_test_app(body, &url, method("GET")).await;
    let tables = app.client.list_tables(&schema).await.unwrap();

    assert_eq!(
        tables.len(),
        2,
        "Expected {} items, got {}",
        2,
        tables.len()
    );
}

#[tokio::test]
async fn list_all_tables() {
    let share = Share {
        name: "share_1".to_string(),
    };
    let body = &format!(
        r#"{{ "items": [ {{ "name":"table_1", "share": "{0}", "schema": "{1}" }}, {{ "name": "table_2", "share": "{0}", "schema": "{1}" }} ] }}"#,
        share.name, "schema_1"
    );
    let url = format!("shares/{}/all-tables", share.name);
    let app = create_mocked_test_app(body, &url, method("GET")).await;
    let tables = app.client.list_all_tables(&share).await.unwrap();

    assert_eq!(
        tables.len(),
        2,
        "Expected {} items, got {}",
        2,
        tables.len()
    );
}

#[tokio::test]
async fn get_table_metadata() {
    let table = Table {
        name: "table_1".to_string(),
        share: "share_1".to_string(),
        schema: "schema_1".to_string(),
    };
    let body = &format!(
        r#"{{ "protocol": {} }}
        {{ "metaData": {} }}"#,
        common::TEST_PROTOCOL_RESPONSE,
        common::TEST_METADATA_RESPONSE,
    );

    let url = format!(
        "shares/{}/schemas/{}/tables/{}/metadata",
        table.share, table.schema, table.name
    );
    let app = create_mocked_test_app(body, &url, method("GET")).await;
    let meta = app.client.get_table_metadata(&table).await.unwrap();

    match meta.protocol {
        Protocol::Delta { .. } => assert!(false, "Wrong protocol deserialization"),
        Protocol::Parquet { min_reader_version } => {
            assert_eq!(min_reader_version, 1, "Protocol mismatch")
        }
    };
    match meta.metadata {
        Metadata::Delta { .. } => assert!(false, "Wrong metadata deserialization"),
        Metadata::Parquet(ParquetMetadata {
            id,
            format,
            name,
            partition_columns,
            configuration,
            ..
        }) => {
            assert_eq!(
                id, "cf9c9342-b773-4c7b-a217-037d02ffe5d8",
                "Metadata ID mismatch"
            );
            assert_eq!(
                format.provider, "parquet",
                "Metadata format provider mismatch"
            );
            assert_eq!(name, None, "Metadata name value should be missing");
            assert_eq!(partition_columns.len(), 0, "There should be no partitions");
            assert_eq!(
                configuration["conf_1_name"], "conf_1_value",
                "Configuration value expected"
            );
        }
    };
}

#[tokio::test]
async fn get_table_version() {
    let table = Table {
        name: "table_1".to_string(),
        share: "share_1".to_string(),
        schema: "schema_1".to_string(),
    };
    let expected_version = "3";
    let url = format!(
        "shares/{}/schemas/{}/tables/{}",
        table.share, table.schema, table.name
    );

    let app = common::create_test_app().await;
    let response =
        ResponseTemplate::new(200).insert_header("delta-table-version", expected_version);

    Mock::given(path(url))
        .and(method("HEAD"))
        .respond_with(response)
        .expect(1)
        .mount(&app.server)
        .await;
    let version = app.client.get_table_version(&table).await;

    assert_eq!(
        &format!("{}", version),
        expected_version,
        "Table version mismatch"
    );
}

#[tokio::test]
async fn list_all_table_files() {
    let table = Table {
        name: "table_1".to_string(),
        share: "share_1".to_string(),
        schema: "schema_1".to_string(),
    };
    let body = &format!(
        r#"{{ "protocol": {} }}
           {{ "metaData": {} }}
           {{ "file": {} }}
           {{ "file": {} }}"#,
        common::TEST_PROTOCOL_RESPONSE,
        common::TEST_METADATA_RESPONSE,
        common::TEST_FILE_RESPONSE,
        common::TEST_FILE_RESPONSE.replace("\"id\": \"1\"", "\"id\": \"2\"")
    );

    let url = format!(
        "shares/{}/schemas/{}/tables/{}/query",
        table.share, table.schema, table.name
    );
    let app = create_mocked_test_app(body, &url, method("POST")).await;
    let files = app.client.list_table_files(&table, None).await.unwrap();

    assert_eq!(files.files.len(), 2, "File count mismatch");
    match &files.files[1] {
        File::Parquet(ParquetFile { id, .. }) => assert_eq!(id, "2", "File id mismatch"),
        File::Delta(DeltaFile { .. }) => assert!(false, "Wrong file deserialization"),
    };
}

#[tokio::test]
async fn get_files() {
    use std::path::Path;

    let table = Table {
        name: "table_1".to_string(),
        share: "share_1".to_string(),
        schema: "schema_1".to_string(),
    };

    let app = common::create_test_app().await;

    // There are fiew pieces here that need to work in sequence
    // 1. List table files
    let list_files_url = format!(
        "shares/{}/schemas/{}/tables/{}/query",
        table.share, table.schema, table.name
    );
    let mut file: ParquetFile =
        serde_json::from_str(common::TEST_FILE_RESPONSE).expect("Invalid file info");
    let file_url_path = "/shares/test.parquet";
    file.url = format!("{}{}", &app.server.uri(), &file_url_path);
    let list_files_body = &format!(
        r#"{{ "protocol": {} }}
           {{ "metaData": {} }}
           {{ "file": {} }}"#,
        common::TEST_PROTOCOL_RESPONSE,
        common::TEST_METADATA_RESPONSE,
        serde_json::to_string(&file).unwrap()
    );
    let list_files_response = ResponseTemplate::new(200).set_body_string(list_files_body);
    Mock::given(path(list_files_url))
        .and(method("POST"))
        .respond_with(list_files_response)
        .expect(1)
        .mount(&app.server)
        .await;

    // 2. Provide the data files for download - use a test Parquet files from the resources
    let parquet_local_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/test/test.parquet");

    let file_content = std::fs::read(parquet_local_path).unwrap();
    let file_response = ResponseTemplate::new(200).set_body_bytes(file_content);
    Mock::given(path(file_url_path))
        .and(method("GET"))
        .respond_with(file_response)
        .expect(1)
        .mount(&app.server)
        .await;

    let mut c = app.client;
    c.data_root = common::get_random_location(Path::new(env!("CARGO_TARGET_TMPDIR")))
        .to_str()
        .unwrap()
        .to_string();

    let expected_path = Path::new(&c.data_root)
        .join(table.fully_qualified_name())
        .join(format!("{}.snappy.parquet", &file.id));

    assert!(!Path::exists(&expected_path), "File should not exist");

    let files = c.get_files(&table, None).await.unwrap();

    assert_eq!(files.len(), 1, "File count mismatch");
    assert_eq!(files[0], expected_path, "File path mismatch");
    assert!(Path::exists(&expected_path), "File should exist");
}

#[tokio::test]
async fn get_dataframe() {
    let table = Table {
        name: "table_1".to_string(),
        share: "share_1".to_string(),
        schema: "schema_1".to_string(),
    };

    let app = common::create_test_app().await;

    // There are fiew pieces here that need to work in sequence
    // 1. List table files
    let list_files_url = format!(
        "shares/{}/schemas/{}/tables/{}/query",
        table.share, table.schema, table.name
    );
    let mut file: ParquetFile =
        serde_json::from_str(common::TEST_FILE_RESPONSE).expect("Invalid file info");
    let file_url_path = "/shares/test.parquet";
    file.url = format!("{}{}", &app.server.uri(), &file_url_path);
    let list_files_body = &format!(
        r#"{{ "protocol": {} }}
           {{ "metaData": {} }}
           {{ "file": {} }}"#,
        common::TEST_PROTOCOL_RESPONSE,
        common::TEST_METADATA_RESPONSE,
        serde_json::to_string(&file).unwrap()
    );
    let list_files_response = ResponseTemplate::new(200).set_body_string(list_files_body);

    // This should be called twice, for both initial call and subsequent call for cached data
    Mock::given(path(list_files_url))
        .and(method("POST"))
        .respond_with(list_files_response)
        .expect(2)
        .mount(&app.server)
        .await;

    // 2. Provide the data files for download - use a test Parquet files from the resources
    let parquet_local_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/test/test.parquet");

    let file_content = std::fs::read(parquet_local_path).unwrap();
    let file_response = ResponseTemplate::new(200).set_body_bytes(file_content);

    // This should only be called once
    Mock::given(path(file_url_path))
        .and(method("GET"))
        .respond_with(file_response)
        .expect(1)
        .mount(&app.server)
        .await;

    let mut c = app.client;
    c.data_root = common::get_random_location(Path::new(env!("CARGO_TARGET_TMPDIR")))
        .to_str()
        .unwrap()
        .to_string();

    let df = c
        .get_dataframe(&table, None)
        .await
        .unwrap()
        .collect()
        .unwrap();
    assert_eq!(df.shape(), (5, 3), "Dataframe shape mismatch");

    // Get the data again, this time it should be served from the local cache (enforced by Expections set on Mocks)
    let df1 = c
        .get_dataframe(&table, None)
        .await
        .unwrap()
        .collect()
        .unwrap();
    assert_eq!(df1.shape(), (5, 3), "Dataframe shape mismatch");
    assert_eq!(
        df1.get_row(0).unwrap().0[1],
        polars::datatypes::AnyValue::String("One"),
        "Row value mismatch"
    );
}
