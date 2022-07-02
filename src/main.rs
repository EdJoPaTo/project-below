use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use ignore::WalkBuilder;

use crate::check_dir_is_project::{check_dir_is_project, Pattern};

mod check_dir_is_project;
mod cli;

fn main() {
    let matches = cli::build().get_matches();

    let patterns = {
        let mut patterns = Vec::new();
        patterns.append(
            &mut matches
                .get_many::<String>("directory")
                .unwrap_or_default()
                .map(String::as_str)
                .map(Pattern::new_directory)
                .collect::<Vec<_>>(),
        );
        patterns.append(
            &mut matches
                .get_many::<String>("file")
                .unwrap_or_default()
                .map(String::as_str)
                .map(Pattern::new_file)
                .collect::<Vec<_>>(),
        );
        patterns
    };
    let base = matches.get_one::<PathBuf>("base").unwrap();
    let recursive = matches.contains_id("recursive");
    let command = matches
        .get_many::<OsString>("command")
        .map(std::iter::Iterator::collect::<Vec<_>>);

    let rx = {
        let (tx, rx) = channel();
        WalkBuilder::new(base)
            .filter_entry(|d| d.file_type().map_or(false, |o| o.is_dir()))
            .build_parallel()
            .run(|| {
                let patterns = patterns.clone();
                let tx = tx.clone();
                Box::new(move |entry| {
                    match entry {
                        Ok(d) => {
                            if let Some(err) = d.error() {
                                eprintln!("Warning for path {:?}: {}", d.path(), err);
                            }
                            if d.depth() == 0 {
                                return ignore::WalkState::Continue;
                            }
                            let path = d.into_path();
                            if check_dir_is_project(&patterns, &path) {
                                tx.send(path).expect("failed to send");
                                if !recursive {
                                    return ignore::WalkState::Skip;
                                }
                            }
                        }
                        Err(err) => eprintln!("Couldn't enter directory {}", err),
                    }
                    ignore::WalkState::Continue
                })
            });
        rx
    };

    for path in rx {
        println!("{}", path.strip_prefix(base).unwrap_or(&path).display());
        // TODO: maybe check path.exists()

        if let Some(command) = &command {
            let start = Instant::now();
            let status = generate_command(command, &path)
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

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / (60 * 60);
    let mut result = String::new();

    if hours > 0 {
        write!(result, "{:>3}h", hours).unwrap();
    } else {
        result += "    ";
    }

    if minutes > 0 {
        write!(result, "{:>2}m", minutes).unwrap();
    } else {
        result += "   ";
    }

    if seconds > 0 {
        write!(result, "{:>2}s", seconds).unwrap();
    } else {
        result += "   ";
    }

    if hours == 0 && minutes == 0 {
        write!(result, "{:>3}ms", duration.subsec_millis()).unwrap();
    }
    result
}
