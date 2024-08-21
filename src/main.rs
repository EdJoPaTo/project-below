use std::ffi::OsString;
use std::fmt::Write;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

use clap::Parser;

use crate::check_dir_is_project::Pattern;

mod check_dir_is_project;
mod cli;
mod walk;

fn main() {
    let matches = cli::Cli::parse();
    let pwd = std::env::current_dir().ok();
    let pwd = pwd.as_deref();

    #[allow(deprecated)]
    if matches.list {
        eprintln!("project-below Hint: --list is no longer required and will be removed in the next major release");
    }

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

    let rx = walk::walk(
        &matches.base_dir,
        patterns,
        matches.hidden,
        matches.recursive,
    );

    for path in rx {
        if !matches.no_harness {
            if matches.canonical {
                print!("{}", path.display());
            } else if matches.relative {
                let relative = pwd.and_then(|pwd| pathdiff::diff_paths(&path, pwd));
                let path = relative.as_ref().unwrap_or(&path);
                print!("{}", path.display());
            } else {
                let path = path.strip_prefix(&matches.base_dir).unwrap_or(&path);
                print!("{}", path.display());
            }

            if matches.print0 {
                print!("\0");
            } else {
                println!();
            }
        }

        if !matches.command.is_empty() {
            let start = Instant::now();
            let status = run_command(&matches.command, &path);
            if !matches.no_harness {
                let took = format_duration(start.elapsed());
                println!("took {took}  {status}\n");
            }
        }
    }
}

fn run_command(raw_command: &[OsString], working_dir: &Path) -> ExitStatus {
    Command::new(&raw_command[0])
        .args(&raw_command[1..])
        .current_dir(working_dir)
        .env("PAGER", "cat")
        .status()
        .unwrap_or_else(|err| panic!("failed to execute process {raw_command:?}: {err}"))
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
