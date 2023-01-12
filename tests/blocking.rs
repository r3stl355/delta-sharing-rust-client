use uuid::Uuid;
use wiremock::matchers::{method, path, MethodExactMatcher};
use wiremock::MockServer;
use wiremock::{Mock, ResponseTemplate};

use delta_sharing::blocking::Client;
use delta_sharing::protocol::*;

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
    let client = Client::new(config, None).unwrap();
    let test_app = BlockingTestApp { client, server };
    test_app
}

fn create_blocking_mocked_test_app(
    body: &str,
    url: &str,
    req_method: MethodExactMatcher,
) -> BlockingTestApp {
    let test_app = create_blocking_test_app();

    let response = ResponseTemplate::new(200).set_body_string(body);
    let mock = Mock::given(path(url))
        .and(req_method)
        .respond_with(response)
        .expect(1)
        .mount(&test_app.server);
    tokio_test::block_on(mock);
    test_app
}

#[test]
fn list_shares() {
    let body = r#"{"items":[{"name":"share_1","id":"1"},{"name":"share_2","id":"2"}]}"#;
    let test_app = create_blocking_mocked_test_app(body, "/shares", method("GET"));
    let shares = test_app.client.list_shares().unwrap();

    assert_eq!(
        shares.len(),
        2,
        "Expected a vector of {}, got {}",
        shares.len(),
        2
    );
}
