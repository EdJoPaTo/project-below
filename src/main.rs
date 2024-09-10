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

    let threads = matches.threads();
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
            threads,
            matches.line_prefix_width,
            matches.no_header,
            matches.result,
        );
        commandpool(threads, &rx, |path| {
            let command = command::Command::new(&matches.command, &path);
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
