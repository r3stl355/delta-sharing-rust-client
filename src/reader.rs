use std::{path::PathBuf};
use polars::prelude::*;
use polars::prelude::{Result as PolarResult};

pub fn load_parquet_files_as_dataframe(parquet_root_dir_path: &PathBuf) -> PolarResult<LazyFrame> {
    let search_pattern = parquet_root_dir_path.join("*.parquet").display().to_string();
    let res = LazyFrame::scan_parquet(search_pattern.into(), Default::default());
    res
}

