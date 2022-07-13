# Delta Sharing client library for Rust

This is a simple library for Rust to access data published via Delta Sharing

## Pre-requisites

- [Delta Sharing](https://databricks.com/product/delta-sharing) set up with at least one shared table 
- Rust installed, e.g. as described [here](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Sample use

1. Create a Rust binary package, e.g. `cargo new delta_sharing_test --bin`

2. Add the following dependencies to `Cargo.toml`

```
delta_sharing = { git = "https://github.com/r3stl355/delta-sharing-rust-client" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
env_logger = "0.9"
```

3. Add a file named `config.json` in the `src` folder next to the `main.rs` file with the following content (replace values within <> with your Delta Share details)
```
{
	"shareCredentialsVersion":1,
	"bearerToken":"<your Delta Share access token>",
	"endpoint":"<your Delta Share endpoinit URL>"
}
```

4. Replace the `main` function with
```
fn main() {
    
    use std::{fs};
    
    env_logger::init();

    let conf_str = &fs::read_to_string("./config.json").unwrap();
    
    let config: delta_sharing::protocol::ProviderConfig = serde_json::from_str(conf_str).expect("Invalid configuration");
    let mut app = delta_sharing::application::Application::new(config, None).unwrap();
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

```

5. Run, e.g. `cargo run --bin main` (alternatively, you can use `RUST_LOG=debug cargo run --bin main` if you want to see some extra debugging information)