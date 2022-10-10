use std::{str::FromStr, string::ToString};

use anyhow::{bail, Error};
use clap::ArgEnum;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub(crate) enum ListQuery {
    // base
    Balance,
    Modules,
    Resources,
    // new
    Resource,
    Events,
}

impl ToString for ListQuery {
    fn to_string(&self) -> String {
        match self {
            // base
            ListQuery::Balance => "balance",
            ListQuery::Modules => "modules",
            ListQuery::Resources => "resources",
            // new
            ListQuery::Resource => "resource",
            ListQuery::Events => "events",
        }
        .to_string()
    }
}

impl FromStr for ListQuery {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // base
            "balance" => Ok(ListQuery::Balance),
            "modules" => Ok(ListQuery::Modules),
            "resources" => Ok(ListQuery::Resources),
            // new
            "events" => Ok(ListQuery::Events),
            "resource" => Ok(ListQuery::Resource),
            _ => bail!("Invalid query. Valid values are modules, resources"),
        }
    }
}
