use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::Paths;

pub(crate) mod table;

// = = =

pub(crate) fn create_template(paths: &Paths) -> Result<()> {
    let Paths {
        sources, project, ..
    } = paths;

    // Move.toml
    fs::copy(sources.join("Move.toml"), project.join("Move.toml"))?;

    // ./sources/
    let mut template_cont: String = all_paths_files(&sources.join("sources"))
        .into_iter()
        .map(content_processing)
        .collect::<Result<Vec<String>>>()?
        .join("\n");
    template_cont = duplicate_constants(template_cont)?;

    template_cont = format!(
        "module self::template {{\n\
            {template_cont}\n\
        }}"
    );

    let project_source_path = project.join("sources");
    fs::create_dir_all(&project_source_path)?;
    fs::write(project_source_path.join("template.move"), template_cont)?;

    Ok(())
}

pub(crate) fn compile_template_mv(paths: &Paths) -> Result<()> {
    Command::new("aptos")
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .current_dir(&paths.project)
        .args(["move", "compile"])
        .output()?;

    let module = paths
        .project
        .join("build")
        .join("intrinsic")
        .join("bytecode_modules")
        .join("template.mv");

    fs::copy(module, &paths.template_mv)
        .map_err(|err| anyhow!("{:?} {err}", &paths.template_mv))?;

    Ok(())
}

// = = =

fn all_paths_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .ok()
        .map(|dir| {
            dir.filter_map(|item| item.ok())
                .map(|item| item.path())
                .flat_map(|path| {
                    if path.is_dir() {
                        all_paths_files(&path)
                    } else if path.is_file() {
                        vec![path]
                    } else {
                        Vec::new()
                    }
                })
                .collect::<Vec<PathBuf>>()
        })
        .unwrap_or_default()
}

fn content_processing(path: PathBuf) -> Result<String> {
    let cont = fs::read_to_string(&path)?;

    let mut result = cont
        .trim()
        .split_once("module ")
        .ok_or_else(|| anyhow!("The module was not found in the file {path:?}"))?
        .1
        .split_once('{')
        .ok_or_else(|| anyhow!("The beginning of the module was not found {path:?}"))?
        .1
        .rsplit_once('}')
        .ok_or_else(|| anyhow!("End of module not found {path:?}"))?
        .0
        .to_string();

    // deleting "use self:: "
    while let Some(start_pos) = result.find("use self::") {
        // use self ..
        let end_pos = result[start_pos..]
            .find(';')
            .ok_or_else(|| anyhow!("No \"use\" ending found {path:?}"))?
            + 1;
        result.replace_range(start_pos..start_pos + end_pos, &" ".repeat(end_pos));

        // #[test_only]
        if let Some(pos) = result[..start_pos].find("#[test_only]") {
            if result[pos + 12..start_pos]
                .chars()
                .all(|t| t.is_ascii_whitespace())
            {
                result.replace_range(pos..start_pos, &" ".repeat(start_pos - pos));
            }
        }
    }

    Ok(result)
}

fn duplicate_constants(mut text: String) -> Result<String> {
    let const_pos = text
        .match_indices("const ")
        .into_iter()
        .map(|(start_pos, ..)| {
            let end_pos = text[start_pos..]
                .find(';')
                .ok_or_else(|| anyhow!("No \"const\" ending found"))?
                + start_pos
                + 1;
            let const_name = text[start_pos..end_pos]
                .trim_start_matches("const")
                .trim()
                .split_once(':')
                .ok_or_else(|| {
                    anyhow!(
                        "Could not get a constant name {}",
                        &text[start_pos..end_pos]
                    )
                })?
                .0;
            Ok((start_pos, end_pos, const_name))
        })
        .collect::<Result<Vec<_>>>()?;

    let need_to_delete: Vec<_> = const_pos
        .iter()
        .enumerate()
        .filter(|(index, (_, _, row))| {
            const_pos[..*index]
                .iter()
                .any(|(_, _, row_back)| row == row_back)
        })
        .map(|(_, (start_pos, end_pos, _))| (*start_pos, *end_pos))
        .collect();

    for (start_pos, end_pos) in need_to_delete {
        text.replace_range(start_pos..end_pos, &" ".repeat(end_pos - start_pos));
    }

    Ok(text)
}
