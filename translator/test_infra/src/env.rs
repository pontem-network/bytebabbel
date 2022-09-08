use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};

pub(crate) fn log_level_filter() -> Result<log::LevelFilter> {
    let env_setting = ["RUST_LOG", "LOGS", "LOG"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .next();

    let filter = match env_setting {
        None => log::LevelFilter::Off,
        Some(data) => log::LevelFilter::from_str(&data)?,
    };

    Ok(filter)
}

pub(crate) fn log_save() -> Result<Option<PathBuf>> {
    let path_dir_str = match std::env::var("LOG_SAVE") {
        Ok(val) => val,
        Err(..) => {
            return Ok(None);
        }
    };
    let result = Some(
        PathBuf::from(&path_dir_str)
            .canonicalize()
            .map_err(|err| anyhow!("{err}\n{path_dir_str}"))?,
    );
    Ok(result)
}
