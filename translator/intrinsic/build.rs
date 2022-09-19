use std::path::PathBuf;
use std::{env, fs};

use anyhow::{anyhow, Result};

mod build_helper;
use build_helper as helper;

#[derive(Debug)]
pub(crate) struct Paths {
    instrinsic_table: PathBuf,
    sources: PathBuf,
    project: PathBuf,
    template_mv: PathBuf,
}

impl Paths {
    pub(crate) fn init() -> Result<Paths> {
        let intrinsic_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);

        let mut project = intrinsic_path.join("../../target/move_tamplate");
        if project.exists() {
            fs::remove_dir_all(&project)
                .map_err(|err| anyhow!("Deleting a project directory {project:?} {err}"))?;
        }
        fs::create_dir_all(&project)
            .map_err(|err| anyhow!("Creating a project directory {project:?} {err}"))?;
        project = project.canonicalize()?;

        let sources = intrinsic_path.join("mv").canonicalize()?;

        let template_mv = intrinsic_path.join("mv").join("template.mv");

        Ok(Paths {
            instrinsic_table: intrinsic_path.join("src").join("table.rs"),
            project,
            sources,
            template_mv,
        })
    }
}

pub fn main() -> Result<()> {
    //  Cargo to "re-run" the build script if the file at the given path has changed
    println!("cargo:rerun-if-changed=mv/sources");

    let paths = Paths::init()?;
    helper::create_template(&paths)?;
    helper::compile_template_mv(&paths)?;
    helper::table::generate_table(&paths)?;

    Ok(())
}
