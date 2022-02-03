use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    let recursive = matches.is_present("recursive");
    let command = matches.values_of_os("command").unwrap().collect::<Vec<_>>();

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
            .filter(
                |d| match check_dir(&directory_patterns, &file_patterns, d.path()) {
                    Ok(true) => true,
                    Ok(false) => false,
                    Err(err) => {
                        eprintln!("Couldn't check directory {}: {}", d.path().display(), err);
                        false
                    }
                },
            )
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

        let start = Instant::now();
        let status = generate_command(&command, path)
            .status()
            .expect("failed to execute process");
        let took = start.elapsed();
        println!("took {}  {}\n", format_duration(took), status);
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
