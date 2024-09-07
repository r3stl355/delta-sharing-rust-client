//! # delta-sharing
//!
//! This crate provides a [Client][client] to access data published via Delta Sharing.
//!
//! Under the hood, it uses HTTP APIs exposed by [Delta Sharing](https://delta.io/sharing/),
//! an open protocol for secure data sharing, making it simple to share data with other organizations
//! regardless of which computing platforms they use.
//!
//! The [delta_sharing::Client][client] is asynchronous. For applications working
//! with smaller datasets, the [delta_sharing::blocking::Client][blocking]
//! may be more convenient.
//!
//! Additional learning resources include:
//!
//! - [Delta Sharing Documentation](https://delta.io/sharing/)
//! - [Delta Sharing GitHub Repo](https://github.com/delta-io/delta-sharing/)
//!
//! ## Optional Features
//!
//! The following [Cargo features][cargo-features] can be enabled:
//!
//! - **blocking**: provides the [blocking][] client.
//!
//! [blocking]: ./blocking/index.html
//! [client]: ./struct.Client.html
//! [cargo-features]: https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section
//!
//! # Quick start example
//!
//! **Note:** This library requires [Delta Sharing](https://delta.io/sharing/) set up and configured, and uses
//! [profile files](https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#profile-file-format)
//!  (which are JSON files containing settings to access a Delta Sharing Server). There are several ways to get started:
//! - Download the profile file to access an open, example Delta Sharing Server hosted by Databricks
//!     [here](https://databricks-datasets-oregon.s3-us-west-2.amazonaws.com/delta-sharing/share/open-datasets.share).
//! - Start your own [Delta Sharing Server](https://github.com/delta-io/delta-sharing#delta-sharing-reference-server)
//!     and create your own profile file following [profile file format](https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#profile-file-format)
//!     to connect to this server.
//! - Download a profile file from your own Delta Sharing data provider (if you have any).
//!
//! When you have your Delta Sharing provider information, replace `"<your Delta Share endpoinit URL>"`
//! and `"<your Delta Share access token>"` values in the example code below with your values.
//!
//!  ```rust
//!  use delta_sharing::Client;
//!  use delta_sharing::protocol::ProviderConfig;
//!  
//!  # async fn run() {
//!  let config = ProviderConfig {
//!      share_credentials_version: 1,
//!      endpoint: "<your Delta Share endpoinit URL>".to_string(),
//!      bearer_token: "<your Delta Share access token>".to_string(),
//!  };
//!  let mut app = Client::new(config, None, None).await.unwrap();
//!  let shares = app.list_shares().await.unwrap();
//!  if shares.len() == 0 {
//!      println!("At least 1 Delta Share is required");
//!  } else {
//!      let tables = app.list_all_tables(&shares[0]).await.unwrap();
//!      if shares.len() == 0 {
//!          println!(
//!             "Need at least one table in share {} (or use a different share)",
//!              shares[0].name
//!          );
//!      } else {
//!          let res = app
//!              .get_dataframe(&tables[0], None)
//!              .await
//!              .unwrap()
//!              .collect()
//!              .unwrap();
//!          println!("Dataframe:\n {}", res);
//!      }
//!  }
//!  # }
//!  ```

#[macro_use]
extern crate log;

pub use self::client::Client;

mod client;
pub mod protocol;
mod reader;
mod utils;

#[cfg(feature = "blocking")]
pub mod blocking;
