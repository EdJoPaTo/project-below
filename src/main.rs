use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ignore::WalkBuilder;

use crate::check_dir_is_project::{check_dir_is_project, Pattern};

mod check_dir_is_project;
mod cli;

fn main() {
    let matches = cli::build().get_matches();

    let pattern = {
        let mut pattern = Vec::new();
        pattern.append(
            &mut matches
                .values_of("directory")
                .unwrap_or_default()
                .map(Pattern::new_directory)
                .collect::<Vec<_>>(),
        );
        pattern.append(
            &mut matches
                .values_of("file")
                .unwrap_or_default()
                .map(Pattern::new_file)
                .collect::<Vec<_>>(),
        );
        pattern
    };
    let recursive = matches.is_present("recursive");
    let command = matches
        .values_of_os("command")
        .map(std::iter::Iterator::collect::<Vec<_>>);

    let already_used: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));

    let walk = {
        let already_used = already_used.clone();
        WalkBuilder::new(".")
            .filter_entry(move |d| {
                let path = d.path();
                if !path.is_dir() {
                    return false;
                }
                if recursive {
                    return true;
                }
                if is_parent_already_checked(&already_used.lock().unwrap(), path) {
                    return false;
                }
                true
            })
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
            .filter(|d| check_dir_is_project(&pattern, d.path()))
    };

    for dir in walk {
        let path = dir.path();
        {
            let mut already_used = already_used.lock().unwrap();
            if !recursive && is_parent_already_checked(&already_used, path) {
                // Should never happen with synchronous walker but maybe a parallel one is introduced in the future
                continue;
            }
            already_used.push(path.to_path_buf());
        }

        println!("{}", path.display());
        // TODO: maybe check path.exists()

        if let Some(command) = &command {
            let start = Instant::now();
            let status = generate_command(command, path)
                .status()
                .expect("failed to execute process");
            let took = start.elapsed();
            println!("took {}  {}\n", format_duration(took), status);
        }
    }
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

fn is_parent_already_checked(already_used: &[PathBuf], path: &Path) -> bool {
    path.ancestors()
        .any(|p| already_used.contains(&p.to_path_buf()))
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / (60 * 60);
    let mut result = String::new();

    if hours > 0 {
        result += &format!("{:>3}h", hours);
    } else {
        result += "    ";
    }

    if minutes > 0 {
        result += &format!("{:>2}m", minutes);
    } else {
        result += "   ";
    }

    if seconds > 0 {
        result += &format!("{:>2}s", seconds);
    } else {
        result += "   ";
    }

    if hours == 0 && minutes == 0 {
        result += &format!("{:>3}ms", duration.subsec_millis());
    }
    result
}
