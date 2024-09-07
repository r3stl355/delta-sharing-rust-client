use delta_sharing::blocking::Client;
use delta_sharing::protocol::ProviderConfig;
use std::fs;

fn main() {
    env_logger::init();

    println!("An example using a blocking client");

    let conf_str = &fs::read_to_string("./config.json").unwrap();

    let config: ProviderConfig = serde_json::from_str(conf_str).expect("Invalid configuration");
    let mut app = Client::new(config, None, None).unwrap();
    let shares = app.list_shares().unwrap();
    if shares.len() == 0 {
        println!("At least 1 Delta Share is required");
    } else {
        let tables = app.list_all_tables(&shares[0]).unwrap();
        if shares.len() == 0 {
            println!(
                "Need at least one table in share {} (or use a different share)",
                shares[0].name
            );
        } else {
            let res = app
                .get_dataframe(&tables[0], None)
                .unwrap()
                .collect()
                .unwrap();
            println!("Dataframe:\n {}", res);
        }
    }
}
