use polars::prelude::*;
use std::path::PathBuf;

pub fn load_parquet_files_as_dataframe(
    parquet_root_dir_path: &PathBuf,
) -> Result<LazyFrame, PolarsError> {
    let search_pattern = parquet_root_dir_path
        .join("*.parquet")
        .display()
        .to_string();
    let res = LazyFrame::scan_parquet(search_pattern, Default::default());
    res
}
