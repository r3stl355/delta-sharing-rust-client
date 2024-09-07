//! A blocking Client.
//!
//! *This requires the optional `blocking` feature to be enabled.*
//!
//! The blocking Client will block the current thread to execute (as opposed to
//! returning futures that need to be executed on a runtime as done by the async Client).
//!
//! The blocking Client should *not* be used within an async runtime, or it will likely
//! panic when attempting to block. If it needs to be used within an async function,
//! consider using an async [Client][crate::Client] instead.
//!
//! The blocking Client has the same features as the async [Client][crate::Client].
//!
//!  # Quick start example
//!
//! **Note:** For provider configuration follow the instructions in the async
//! [Client][crate::Client] docs.
//!
//!  ```rust
//!  use delta_sharing::blocking::Client;
//!  use delta_sharing::protocol::ProviderConfig;
//!  
//!  # fn run() {
//!  let config = ProviderConfig {
//!      share_credentials_version: 1,
//!      endpoint: "<your Delta Share endpoinit URL>".to_string(),
//!      bearer_token: "<your Delta Share access token>".to_string(),
//!  };
//!  let mut app = Client::new(config, None, None).unwrap();
//!  let shares = app.list_shares().unwrap();
//!  if shares.len() == 0 {
//!      println!("At least 1 Delta Share is required");
//!  } else {
//!      let tables = app.list_all_tables(&shares[0]).unwrap();
//!      if shares.len() == 0 {
//!          println!(
//!             "You need to have at least one table in share {}, or use a different share",
//!              shares[0].name
//!          );
//!      } else {
//!          let res = app.get_dataframe(&tables[0], None).unwrap().collect().unwrap();
//!          println!("Dataframe:\n {}", res);
//!      }
//!  }
//!  # }
//!  ```

pub use self::client::Client;

mod client;
