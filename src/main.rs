use std::env;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;

const SCRIPT_REGION: &str = "Script";

enum ScriptCopyError {
    CouldNotOpenSourceFile,
    CouldNotCreateTargetDir(String),
    CouldNotOpenTargetFile,
    IoError(io::Error),
}

fn get_scripts_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").expect("env var APPDATA should be set on Windows");
    let s = format!("{appdata}\\SpaceEngineers\\IngameScripts\\local");
    PathBuf::from(s)
}

fn get_project_name() -> Option<String> {
    Some(
        env::current_dir()
            .ok()?
            .file_name()?
            .to_string_lossy()
            .to_string(),
    )
}

fn copy_script<S: AsRef<Path>, T: AsRef<Path>>(
    source_path: S,
    target_path: T,
) -> Result<(), ScriptCopyError> {
    let source_file = BufReader::new(
        fs::File::open(source_path).map_err(|_| ScriptCopyError::CouldNotOpenSourceFile)?,
    );

    let target_dir = target_path
        .as_ref()
        .parent()
        .expect("target path should have at least 2 levels");

    if !target_dir.exists() {
        fs::create_dir_all(target_dir).map_err(|_| {
            ScriptCopyError::CouldNotCreateTargetDir(
                target_dir
                    .to_str()
                    .expect("path should be valid UTF-8")
                    .to_owned(),
            )
        })?;
    };

    let mut target_file = BufWriter::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(target_path)
            .map_err(|_| ScriptCopyError::CouldNotOpenTargetFile)?,
    );

    let mut regions: Vec<String> = Vec::new();
    let mut in_content = false;
    let mut content_indent = 0;

    for line in source_file.lines().flatten() {
        let trimmed = line.trim();
        if trimmed.starts_with("#region") {
            let region = &trimmed[8..];
            regions.push(region.to_string());
            if region == SCRIPT_REGION {
                in_content = true;
                content_indent = line
                    .find('#')
                    .expect("line should contain # because of the starts_with check above");
            }
            continue;
        } else if trimmed.starts_with("#endregion") {
            regions.pop().map_or_else(
                || {
                    println!("Mismatched regions");
                },
                |region| {
                    if region == SCRIPT_REGION {
                        in_content = false;
                    }
                },
            );
            continue;
        }

        if in_content {
            if line.len() > content_indent {
                target_file
                    .write_all(line[content_indent..].as_bytes())
                    .map_err(ScriptCopyError::IoError)?;
            }
            target_file
                .write_all(b"\n")
                .map_err(ScriptCopyError::IoError)?;
        }
    }

    Ok(())
}

fn main() {
    let Some(project_name) = get_project_name() else {
        println!("Could not determine script name. Is the current directory readable?");
        return;
    };
    let mut scripts_path = get_scripts_path();
    scripts_path.push(project_name.as_str());

    let mut target_path = scripts_path.clone();
    target_path.push("script.cs");

    let Ok(mut source_path) = env::current_dir() else {
        println!("Could not determine working directory");
        return;
    };
    source_path.push("Script.cs");

    match copy_script(&source_path, &target_path) {
        Ok(_) => println!(
            "Script written to {}",
            scripts_path.to_str().expect("path should be valid UTF-8")
        ),
        Err(ScriptCopyError::CouldNotCreateTargetDir(path)) => {
            println!("Could not create target directory {path}");
        }
        Err(ScriptCopyError::CouldNotOpenTargetFile) => println!("Could not open target file"),
        Err(ScriptCopyError::CouldNotOpenSourceFile) => {
            println!("Could not open Script.cs in this directory");
        }
        Err(ScriptCopyError::IoError(_)) => println!("I/O error"),
    }
}
