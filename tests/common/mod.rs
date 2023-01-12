use uuid::Uuid;
use wiremock::MockServer;
use wiremock::{Mock, ResponseTemplate};

use wiremock::matchers::{path, MethodExactMatcher};

use delta_sharing::protocol::*;
use delta_sharing::Client;

pub struct TestApp {
    pub client: Client,
    pub server: MockServer,
}

pub async fn create_test_app() -> TestApp {
    let _ = env_logger::try_init();

    // Launch a mock server
    let server = MockServer::start().await;
    let config = ProviderConfig {
        share_credentials_version: 1,
        endpoint: server.uri(),
        bearer_token: Uuid::new_v4().to_string(),
    };
    let client = Client::new(config, None).await.unwrap();
    let test_app = TestApp { client, server };
    test_app
}

pub async fn create_mocked_test_app(
    body: &str,
    url: &str,
    req_method: MethodExactMatcher,
) -> TestApp {
    let test_app = create_test_app().await;

    let response = ResponseTemplate::new(200).set_body_string(body);
    Mock::given(path(url))
        .and(req_method)
        .respond_with(response)
        .expect(1)
        .mount(&test_app.server)
        .await;

    // Even though only `client` will likely be needed, cannot return just the client otherwise `expect` validation will fail. Alternative
    // is to return the `client` only but don't set the `expect` on Mock above
    test_app
}
