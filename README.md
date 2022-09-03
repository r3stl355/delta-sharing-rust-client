[![main](https://github.com/r3stl355/delta-sharing-rust-client/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/r3stl355/delta-sharing-rust-client/actions/workflows/main.yml)

# Delta Sharing client library for Rust

This is a simple library for Rust to access data published via Delta Sharing

## Features

- Retrieve Delta Sharing information (shares, schemas, tables and files)
- Query shared table data using [Polars](https://pola-rs.github.io/polars/polars/index.html). `get_dataframe` downloads the table's parquet files (and caches then locally for subsequent queries) and returns a lazy abstraction (logical plan) over an eager DataFrame. This lazy abstraction provides methods for incrementally modifying that logical plan until output is requested (via `collect`).

## Pre-requisites

- [Delta Sharing](https://databricks.com/product/delta-sharing) set up with at least one shared table 
- Rust is installed, e.g. as described [here](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Sample use

Use this [sample project](https://github.com/r3stl355/delta-sharing-rust-cllient-use-example) for a quick start

## TODO

- move to `async` mode (e.g. swap blocking `reqwest` Client to async version)
- write more tests