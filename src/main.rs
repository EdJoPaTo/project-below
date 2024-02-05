use std::ffi::OsStr;
use std::fmt::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use clap::Parser;
use ignore::WalkBuilder;

use crate::check_dir_is_project::{check_dir_is_project, Pattern};

mod check_dir_is_project;
mod cli;

fn main() {
    let matches = cli::Cli::parse();
    let cli::Cli {
        base_dir: base,
        command,
        ..
    } = matches;
    let pwd = std::env::current_dir().ok();
    let pwd = pwd.as_deref();

    let patterns = {
        let mut patterns = Vec::new();
        patterns.append(
            &mut matches
                .directory
                .into_iter()
                .map(Pattern::new_directory)
                .collect::<Vec<_>>(),
        );
        patterns.append(
            &mut matches
                .file
                .into_iter()
                .map(Pattern::new_file)
                .collect::<Vec<_>>(),
        );
        patterns
    };

    let rx = {
        let (tx, rx) = channel();
        WalkBuilder::new(&base)
            .hidden(!matches.hidden)
            .filter_entry(|d| d.file_type().map_or(false, |o| o.is_dir()))
            .build_parallel()
            .run(|| {
                let patterns = patterns.clone();
                let tx = tx.clone();
                Box::new(move |entry| {
                    match entry {
                        Ok(d) => {
                            if let Some(err) = d.error() {
                                eprintln!("Warning for path {:?}: {err}", d.path());
                            }
                            if d.depth() == 0 {
                                return ignore::WalkState::Continue;
                            }
                            let path = d.into_path();
                            if check_dir_is_project(&patterns, &path) {
                                tx.send(path).expect("failed to send");
                                if !matches.recursive {
                                    return ignore::WalkState::Skip;
                                }
                            }
                        }
                        Err(err) => eprintln!("Couldn't enter directory {err}"),
                    }
                    ignore::WalkState::Continue
                })
            });
        rx
    };

    for path in rx {
        if matches.canonical {
            print!("{}", path.display());
        } else if matches.relative {
            let relative = pwd.and_then(|pwd| pathdiff::diff_paths(&path, pwd));
            let p = relative.as_ref().unwrap_or(&path);
            print!("{}", p.display());
        } else {
            let p = path.strip_prefix(&base).unwrap_or(&path);
            print!("{}", p.display());
        }

        if matches.print0 {
            print!("\0");
        } else {
            println!();
        }

        if !command.is_empty() {
            let start = Instant::now();
            let status = generate_command(&command, &path)
                .status()
                .unwrap_or_else(|err| panic!("failed to execute process {command:?}: {err}"));
            let took = format_duration(start.elapsed());
            println!("took {took}  {status}\n");
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
        write!(result, "{hours:>3}h").unwrap();
    } else {
        result += "    ";
    }

    if minutes > 0 {
        write!(result, "{minutes:>2}m").unwrap();
    } else {
        result += "   ";
    }

    if seconds > 0 {
        write!(result, "{seconds:>2}s").unwrap();
    } else {
        result += "   ";
    }

    if hours == 0 && minutes == 0 {
        write!(result, "{:>3}ms", duration.subsec_millis()).unwrap();
    }
    result
}
