use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

use clap::Parser;

use crate::check_dir_is_project::Pattern;

mod check_dir_is_project;
mod cli;
mod display;
mod walk;

fn main() {
    let matches = cli::Cli::parse();

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

    let display = display::PathStyle::new(matches.canonical, matches.relative, matches.base_dir);

    for path in rx {
        if !matches.no_harness {
            if matches.print0 {
                print!("{}\0", display.path(&path));
            } else {
                println!("{}", display.path(&path));
            }
        }

        if !matches.command.is_empty() {
            let (status, took) = run_command(&matches.command, &path);
            if !matches.no_harness {
                display.print_endline(&path, took, status);
            }
        }
    }
}

fn run_command(raw_command: &[OsString], working_dir: &Path) -> (ExitStatus, Duration) {
    let start = Instant::now();
    let status = Command::new(&raw_command[0])
        .args(&raw_command[1..])
        .current_dir(working_dir)
        .env("PAGER", "cat")
        .status()
        .unwrap_or_else(|err| panic!("failed to execute process {raw_command:?}: {err}"));
    let took = start.elapsed();
    (status, took)
}
