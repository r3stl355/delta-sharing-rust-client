use wiremock::MockServer;
use uuid::Uuid;

use delta_sharing::application::Application;
use delta_sharing::protocol::*;

pub struct TestApp {
    pub app: Application,
    pub server: MockServer,
}

pub async fn spawn_test_app() -> TestApp {

    // let _ = env_logger::builder().is_test(true).try_init();
    env_logger::init();

    // Launch a mock server
    let server = MockServer::start().await;

    let token = Uuid::new_v4().to_string();
    let config = ProviderConfig {
        share_credentials_version: 1,
        endpoint: server.uri(),
        bearer_token: token,
    };
    let app = Application::new(config, None).unwrap();
    let test_app = TestApp {
        app,
        server,
    };
    test_app
}

