use std::env::VarError;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use crossbeam_channel::Receiver;

use crate::check_dir_is_project::Pattern;
use crate::cli::CommandOutput;

mod byte_lines;
mod check_dir_is_project;
mod cli;
mod command;
mod harness;
mod path_style;
mod shortened_path;
mod took;
mod walk;

fn main() {
    let matches = cli::Cli::get();

    let (concurrent_commands, jobs_per_command) = matches.concurrency();
    let patterns = Pattern::many(matches.directory, matches.file);

    let rx = walk::walk(
        &matches.base_dir,
        patterns,
        matches.hidden,
        matches.recursive,
    );

    let path_style = path_style::PathStyle::new(matches.path_style, matches.base_dir);

    if matches.command.is_empty() {
        for path in rx {
            if matches.print0 {
                print!("{}\0", path_style.path(&path));
            } else {
                println!("{}", path_style.path(&path));
            }
        }
    } else {
        let harness = harness::Config::new(
            path_style,
            concurrent_commands,
            matches.line_prefix_width,
            matches.no_header,
            matches.result,
        );

        let mut commandenvs = Vec::new();
        if let Some(jobs) = jobs_per_command {
            commandenvs.push(("CARGO_BUILD_JOBS", jobs.to_string()));

            let key = "MAKEFLAGS";
            let value = match std::env::var(key) {
                Ok(makeflags) => format!("{makeflags} -j{jobs}"),
                Err(VarError::NotPresent) => format!("-j{jobs}"),
                Err(VarError::NotUnicode(_)) => panic!("MAKEFLAGS are not valid unicode"),
            };
            commandenvs.push((key, value));
        }

        commandpool(concurrent_commands, &rx, |path| {
            let command = command::Command::new(&matches.command, &path, &commandenvs);
            let harness = harness.create(&path);
            match matches.output {
                CommandOutput::Inherit => {
                    harness.inherit_header();
                    let (status, took) = command.inherit();
                    harness.result(took, status);
                }
                CommandOutput::LinePrefix => {
                    let (status, took) = command.lineprefixed(&harness.line_prefix());
                    harness.result(took, status);
                }
                CommandOutput::Collect => {
                    let (output, took) = command.output();
                    let _stdout = std::io::stdout().lock();
                    harness.collect(&output);
                    harness.result(took, output.status);
                }
                CommandOutput::Null => {
                    let (status, took) = command.null();
                    harness.result(took, status);
                }
            }
        });
    }
}

fn commandpool<'scope, F>(threads: NonZeroUsize, rx: &Receiver<PathBuf>, func: F)
where
    F: Fn(PathBuf) + Send + Sync + 'scope,
{
    std::thread::scope(|scope| {
        for _ in 1..threads.get() {
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
        for path in rx {
            func(path);
        }
    });
}
