# Delta Sharing client library for Rust

This is a simple library for Rust to access data published via Delta Sharing. Has an async (`delta-sharing::application::Application`) version, and also a blocking (`delta-sharing::blocking::Application`) version for smaller operations, both exposing similar APIs.

## Features

- Retrieve Delta Sharing information (shares, schemas, tables and files)
- Query shared table data using [Polars](https://pola-rs.github.io/polars/polars/index.html). `get_dataframe` downloads the table's parquet files (and caches then locally for subsequent queries) and returns a lazy abstraction (logical plan) over an eager DataFrame. This lazy abstraction provides methods for incrementally modifying that logical plan until output is requested (via `collect`).

## Pre-requisites

- [Delta Sharing](https://databricks.com/product/delta-sharing) set up with at least one shared table 
- Rust is installed, e.g. as described [here](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Quick start

- Clone this repo
- Set the `bearerToken` and `endpoint` values in the `config.json` to match your Delta Sharing information
- Run a simple example included with the library: `cargo run --example async`. This example is using an async version of the library. When executed, it will get and display all the data from the first Data Sharing table it finds. For an example using a blocking version of the client, run `cargo run --example blocking --features="delta-sharing/blocking"`

## TODO

Need to add more tests