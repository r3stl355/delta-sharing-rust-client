fn main() {
    
    use std::{fs};
    
    env_logger::init();

    println!("An example using a blocking client");

    let conf_str = &fs::read_to_string("./config.json").unwrap();
    
    let config: delta_sharing::protocol::ProviderConfig = serde_json::from_str(conf_str).expect("Invalid configuration");
    let mut app = delta_sharing::blocking::Application::new(config, None).unwrap();
    let shares = app.list_shares().unwrap();
    if shares.len() == 0 {
        println!("At least 1 Delta Share is required");
    } else {
        let tables = app.list_all_tables(&shares[0]).unwrap();
        if shares.len() == 0 {
            println!("You need to have at least one table in share {}, or use a different share", shares[0].name);
        } else {
            let res = app.get_dataframe(&tables[0]).unwrap().collect().unwrap();
            println!("Dataframe:\n {}", res);
        }
    }
}