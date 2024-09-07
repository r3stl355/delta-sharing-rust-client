use std::path::{Path, PathBuf};

use uuid::Uuid;
use wiremock::MockServer;
use wiremock::{Mock, ResponseTemplate};

use wiremock::matchers::{path, MethodExactMatcher};

use delta_sharing::protocol::*;
use delta_sharing::Client;

#[allow(dead_code)]
pub struct TestApp {
    pub client: Client,
    pub server: MockServer,
}

pub const TEST_PROTOCOL_RESPONSE: &str = r#"{ "minReaderVersion": 1 }"#;
pub const TEST_METADATA_RESPONSE: &str = r#"{ "id": "cf9c9342-b773-4c7b-a217-037d02ffe5d8", "format": { "provider": "parquet" }, "schemaString": "{\"type\":\"struct\",\"fields\":[{\"name\":\"int_field_1\",\"type\":\"integer\",\"nullable\":true,\"metadata\":{}},{\"name\":\"double_field_1\",\"type\":\"double\",\"nullable\":true,\"metadata\":{}}]}", "partitionColumns": [], "configuration": {"conf_1_name": "conf_1_value"} }"#;
pub const TEST_FILE_RESPONSE: &str = r#"{ "url": "<url>", "id": "1", "partitionValues": {}, "size": 2350, "stats": "{\"numRecords\":1}" }"#;

#[allow(dead_code)]
pub async fn create_test_app() -> TestApp {
    let _ = env_logger::try_init();

    // Launch a mock server
    let server = MockServer::start().await;
    let config = ProviderConfig {
        share_credentials_version: 1,
        endpoint: server.uri(),
        bearer_token: Uuid::new_v4().to_string(),
    };
    let client = Client::new(config, None, None).await.unwrap();
    let app = TestApp { client, server };
    app
}

#[allow(dead_code)]
pub async fn create_mocked_test_app(
    body: &str,
    url: &str,
    req_method: MethodExactMatcher,
) -> TestApp {
    let app = create_test_app().await;

    let response = ResponseTemplate::new(200).set_body_string(body);
    Mock::given(path(url))
        .and(req_method)
        .respond_with(response)
        .expect(1)
        .mount(&app.server)
        .await;

    // Even though only `client` will likely be needed, cannot return just the client otherwise `expect` validation will fail. Alternative
    // is to return the `client` only but don't set the `expect` on Mock above
    app
}

pub fn get_random_location(root: &Path) -> PathBuf {
    use rand::distributions::{Alphanumeric, DistString};

    let r = &mut rand::thread_rng();
    let mut p = root.join(&Alphanumeric.sample_string(r, 10));
    while Path::exists(&p) {
        p = root.join(Alphanumeric.sample_string(r, 10));
    }
    return p;
}
