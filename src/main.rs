use std::env;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;

const SCRIPT_REGION: &str = "Script";

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

fn copy_script<S: AsRef<Path>, T: AsRef<Path>>(source_path: S, target_path: T) -> io::Result<()> {
    let source_file = BufReader::new(fs::File::open(source_path)?);

    let target_dir = target_path
        .as_ref()
        .parent()
        .expect("target path should have at least 2 levels");

    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?
    };

    let mut target_file = BufWriter::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(target_path)?,
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
                // line must contain a # because of the starts_with check above
                content_indent = line.find('#').unwrap();
            }
            continue;
        } else if trimmed.starts_with("#endregion") {
            if let Some(region) = regions.pop() {
                if region == SCRIPT_REGION {
                    in_content = false;
                }
            } else {
                println!("Mismatched regions");
            }
            continue;
        }

        if in_content {
            if line.len() > content_indent {
                target_file.write_all(line[content_indent..].as_bytes())?;
                target_file.write_all(b"\n")?;
            } else {
                target_file.write_all(b"\n")?;
            }
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
        Ok(_) => println!("Script written to {}", scripts_path.to_str().unwrap()),
        Err(_) => println!("Could not copy script."),
    }
}
