use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;

use crate::check_dir_is_project::Pattern;

mod check_dir_is_project;
mod cli;
mod display;
mod walk;

fn main() {
    let matches = cli::Cli::get();

    let threads = matches.threads();
    let patterns = Pattern::many(matches.directory, matches.file);

    let rx = walk::walk(
        &matches.base_dir,
        patterns,
        matches.hidden,
        matches.recursive,
    );

    let display = display::PathStyle::new(matches.canonical, matches.relative, matches.base_dir);

    if matches.command.is_empty() {
        for path in rx {
            if matches.print0 {
                print!("{}\0", display.path(&path));
            } else {
                println!("{}", display.path(&path));
            }
        }
    } else {
        commandpool(threads, &rx, |path| {
            if !matches.no_header {
                println!("{}", display.path(&path));
            }
            let (status, took) = run_command(&matches.command, &path);
            if matches.result.print(status.success()) {
                display.print_endline(&path, took, status);
            }
        });
    }
}

fn commandpool<'scope, F>(threads: NonZeroUsize, rx: &Receiver<PathBuf>, func: F)
where
    F: Fn(PathBuf) + Send + Sync + 'scope,
{
    std::thread::scope(|scope| {
        for _ in 0..threads.get() {
            let rx = rx.clone();
            std::thread::Builder::new()
                .name("commandpool".to_owned())
                .spawn_scoped(scope, || {
                    for path in rx {
                        func(path);
                    }
                })
                .expect("failed to spawn thread");
        }
    });
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
