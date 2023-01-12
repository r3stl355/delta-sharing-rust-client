mod common;

use common::create_mocked_test_app;
use delta_sharing::protocol::*;
use wiremock::matchers::method;

#[cfg(not(feature = "blocking"))]
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
        shares.len(),
        2
    );
}

#[cfg(not(feature = "blocking"))]
#[tokio::test]
async fn list_schemas() {
    let share_name = "share_1";
    let share = Share {
        name: share_name.to_string(),
    };
    let body = &format!(
        r#"{{ "items": [ {{ "name":"schema_1", "share": "{}" }}, {{ "name": "schema_2", "share": "{}" }} ] }}"#,
        share_name, share_name
    );
    let app = create_mocked_test_app(
        body,
        &format!("/shares/{}/schemas", share_name),
        method("GET"),
    )
    .await;

    let shares = app.client.list_schemas(&share).await.unwrap();

    assert_eq!(
        shares.len(),
        2,
        "Expected {} items, got {}",
        shares.len(),
        2
    );
}
