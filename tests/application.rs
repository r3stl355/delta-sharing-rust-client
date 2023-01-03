mod common;

use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use tokio_test::block_on;
use common::spawn_test_app;

#[test]
fn parse_share_listing() {
    // Arrange
    let test_app = block_on(spawn_test_app());
    let body = r#"{"items":[{"name":"share_1","id":"1"},{"name":"share_2","id":"2"}]}"#;
    let response = ResponseTemplate::new(200).set_body_string(body);
    let mock = Mock::given(path("/shares"))
        .and(method("GET"))
        .respond_with(response)
        .expect(1)
        .mount(&test_app.server);
    block_on(mock);
    
    // Act
    let shares = block_on(test_app.app.list_shares()).unwrap();

    // Assert
    assert_eq!(shares.len(), 2, "Expected a vector of {}, got {}", shares.len(), 2);
}

