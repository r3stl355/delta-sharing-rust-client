mod common;

use delta_sharing::blocking::Client;
use delta_sharing::protocol::*;
use std::path::Path;
use uuid::Uuid;
use wiremock::matchers::{method, path, MethodExactMatcher};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct BlockingTestApp {
    pub client: Client,
    pub server: MockServer,
}

fn create_blocking_test_app() -> BlockingTestApp {
    let _ = env_logger::try_init();

    let server = tokio_test::block_on(MockServer::start());
    let config = ProviderConfig {
        share_credentials_version: 1,
        endpoint: server.uri(),
        bearer_token: Uuid::new_v4().to_string(),
    };
    let client = Client::new(config, None, None).unwrap();
    let test_app = BlockingTestApp { client, server };
    test_app
}

fn create_blocking_mocked_test_app(
    body: &str,
    url: &str,
    req_method: MethodExactMatcher,
) -> BlockingTestApp {
    let app = create_blocking_test_app();

    let response = ResponseTemplate::new(200).set_body_string(body);
    let mock = Mock::given(path(url))
        .and(req_method)
        .respond_with(response)
        .expect(1)
        .mount(&app.server);
    tokio_test::block_on(mock);
    app
}

#[test]
fn list_shares() {
    let body = r#"{"items":[{"name":"share_1","id":"1"},{"name":"share_2","id":"2"}]}"#;
    let app = create_blocking_mocked_test_app(body, "/shares", method("GET"));
    let shares = app.client.list_shares().unwrap();

    assert_eq!(
        shares.len(),
        2,
        "Expected a vector of {}, got {}",
        2,
        shares.len(),
    );
}

#[test]
fn get_dataframe() {
    let table = Table {
        name: "table_1".to_string(),
        share: "share_1".to_string(),
        schema: "schema_1".to_string(),
    };

    let app = create_blocking_test_app();

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
    let m = Mock::given(path(list_files_url))
        .and(method("POST"))
        .respond_with(list_files_response)
        .expect(2)
        .mount(&app.server);
    tokio_test::block_on(m);

    // 2. Provide the data files for download - use a test Parquet files from the resources
    let parquet_local_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/test/test.parquet");

    let file_content = std::fs::read(parquet_local_path).unwrap();
    let file_response = ResponseTemplate::new(200).set_body_bytes(file_content);

    // This should only be called once
    let m = Mock::given(path(file_url_path))
        .and(method("GET"))
        .respond_with(file_response)
        .expect(1)
        .mount(&app.server);
    tokio_test::block_on(m);

    let mut c = app.client;
    c.data_root = common::get_random_location(Path::new(env!("CARGO_TARGET_TMPDIR")))
        .to_str()
        .unwrap()
        .to_string();

    let df = c.get_dataframe(&table, None).unwrap().collect().unwrap();
    assert_eq!(df.shape(), (5, 3), "Dataframe shape mismatch");

    // Get the data again, this time it should be served from the local cache (enforced by Expections set on Mocks)
    let df1 = c.get_dataframe(&table, None).unwrap().collect().unwrap();
    assert_eq!(df1.shape(), (5, 3), "Dataframe shape mismatch");
    assert_eq!(
        df1.get_row(0).unwrap().0[1],
        polars::datatypes::AnyValue::String("One"),
        "Row value mismatch"
    );
}
