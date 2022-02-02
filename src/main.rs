use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use glob::Pattern;
use ignore::WalkBuilder;

mod cli;

fn main() {
    let matches = cli::build().get_matches();

    let directory_patterns = matches
        .values_of("directory")
        .unwrap_or_default()
        .map(|s| Pattern::new(s).unwrap())
        .collect::<Vec<_>>();
    let file_patterns = matches
        .values_of("file")
        .unwrap_or_default()
        .map(|s| Pattern::new(s).unwrap())
        .collect::<Vec<_>>();
    let command = matches.values_of_os("command").unwrap().collect::<Vec<_>>();

    let walk = WalkBuilder::new(".")
        .filter_entry(move |d| d.path().is_dir())
        .build()
        .filter_map(|d| match d {
            Ok(d) => {
                if let Some(err) = d.error() {
                    eprintln!("Warning for path {}: {}", d.path().display(), err);
                }
                Some(d)
            }
            Err(err) => {
                eprintln!("Couldn't enter directory {}", err);
                None
            }
        })
        .filter(
            |d| match check_dir(&directory_patterns, &file_patterns, d.path()) {
                Ok(true) => true,
                Ok(false) => false,
                Err(err) => {
                    eprintln!("Couldn't check directory {}: {}", d.path().display(), err);
                    false
                }
            },
        );

    let mut already_used: Vec<PathBuf> = Vec::new();

    for dir in walk {
        let path = dir.path();

        if matches.is_present("prune") && is_parent_directory_already_checked(&already_used, path) {
            // TODO: it would be better to include something like this in `filter_entry`.
            // But this requires Send so Arc & Mutex are probably required.
            continue;
        }
        already_used.push(path.to_path_buf());

        println!("{}", path.display());
        // TODO: maybe check path.exists()

        let status = generate_command(&command, path)
            .status()
            .expect("failed to execute process");
        println!("{}", status);
    }
}

fn check_dir(
    directory_patterns: &[Pattern],
    file_patterns: &[Pattern],
    dir: &Path,
) -> std::io::Result<bool> {
    let entries = dir.read_dir()?.filter_map(Result::ok).collect::<Vec<_>>();
    let dirs = entries
        .iter()
        .filter(|d| d.path().is_dir())
        .filter_map(|s| s.file_name().to_str().map(std::string::ToString::to_string))
        .collect::<Vec<_>>();
    let files = entries
        .iter()
        .filter(|d| d.path().is_file())
        .filter_map(|s| s.file_name().to_str().map(std::string::ToString::to_string))
        .collect::<Vec<_>>();

    let dirs_match = directory_patterns
        .iter()
        .all(|p| dirs.iter().any(|d| p.matches(d)));
    let files_match = file_patterns
        .iter()
        .all(|p| files.iter().any(|f| p.matches(f)));

    Ok(dirs_match && files_match)
}

fn generate_command<C, S>(command: C, working_dir: &Path) -> Command
where
    C: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = command.into_iter();
    let program = command.next().unwrap();
    let mut c = Command::new(program);
    c.args(command).current_dir(working_dir).env("PAGER", "cat");
    c
}

fn is_parent_directory_already_checked(checked: &[PathBuf], path: &Path) -> bool {
    let mut p = path;
    while let Some(parent) = p.parent() {
        if checked.contains(&parent.to_path_buf()) {
            return true;
        }
        p = parent;
    }
    false
}
