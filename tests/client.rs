mod common;

use common::create_mocked_test_app;
use delta_sharing::protocol::*;
use wiremock::matchers::method;

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

    let body = r#"{ "protocol":{ "minReaderVersion": 1 } }
                        { "metaData": { "id": "cf9c9342-b773-4c7b-a217-037d02ffe5d8", "format": { "provider": "parquet" }, "schemaString": "{\"type\":\"struct\",\"fields\":[{\"name\":\"pickup_zip\",\"type\":\"integer\",\"nullable\":true,\"metadata\":{}},{\"name\":\"max_trip_distance\",\"type\":\"double\",\"nullable\":true,\"metadata\":{}}]}", "partitionColumns": [], "configuration": {"conf_1_name": "conf_1_value"}}}"#;

    let url = format!(
        "shares/{}/schemas/{}/tables/{}/metadata",
        table.share, table.schema, table.name
    );
    let app = create_mocked_test_app(body, &url, method("GET")).await;

    let meta = app.client.get_table_metadata(&table).await.unwrap();

    assert_eq!(meta.protocol.min_reader_version, 1, "Protocol mismatch");
    assert_eq!(
        meta.metadata.id, "cf9c9342-b773-4c7b-a217-037d02ffe5d8",
        "Metadata ID mismatch"
    );
    assert_eq!(
        meta.metadata.format.provider, "parquet",
        "Metadata format provider mismatch"
    );
    assert_eq!(
        meta.metadata.name, None,
        "Metadata name value should be missing"
    );
    assert_eq!(
        meta.metadata.partition_columns.len(),
        0,
        "There should be no partitions"
    );
    assert_eq!(
        meta.metadata.configuration["conf_1_name"], "conf_1_value",
        "Configuration value expected"
    );
}
