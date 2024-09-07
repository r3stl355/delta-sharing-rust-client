use delta_sharing::protocol::ProviderConfig;
use delta_sharing::Client;

#[tokio::main]
async fn main() {
    use std::fs;

    env_logger::init();

    println!("An example using an async client");

    let conf_str = &fs::read_to_string("./config.json").unwrap();
    let config: ProviderConfig = serde_json::from_str(conf_str).expect("Invalid configuration");
    let mut app = Client::new(config, None, None).await.unwrap();
    let shares = app.list_shares().await.unwrap();
    if shares.len() == 0 {
        println!("At least 1 Delta Share is required");
    } else {
        let share_name = &shares[0].name;
        println!(
            "Found {} shares, exploring share [{}]",
            shares.len(),
            share_name
        );
        let schemas = app.list_schemas(&shares[0]).await.unwrap();
        println!("Found {} schemas in share [{}]", schemas.len(), &share_name);

        if schemas.len() == 0 {
            let schema_tables = app.list_tables(&schemas[0]).await.unwrap();
            println!(
                "Found {} tables in schema [{}]",
                schema_tables.len(),
                &schemas[0].name
            );
        }

        let tables = app.list_all_tables(&shares[0]).await.unwrap();
        if shares.len() == 0 {
            println!(
                "Need at least one table in share {} (or use a different share)",
                shares[0].name
            );
        } else {
            let res = app
                .get_dataframe(&tables[0], None)
                .await
                .unwrap()
                .collect()
                .unwrap();
            println!("Dataframe:\n {}", res);
        }
    }
}
