use std::num::NonZeroUsize;
use std::path::PathBuf;

use cli::CommandOutput;
use crossbeam_channel::Receiver;

use crate::check_dir_is_project::Pattern;

mod byte_lines;
mod check_dir_is_project;
mod cli;
mod command;
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

    let display = display::PathStyle::new(matches.path_style, matches.base_dir);

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
            let command = command::Command::new(&matches.command, &path);
            let (status, took) = match matches.output {
                CommandOutput::Inherit => command.inherit(),
                CommandOutput::LinePrefix => command.lineprefixed(&format!(
                    "{:width$}  ",
                    display.path(&path),
                    width = matches.line_prefix_width
                )),
                CommandOutput::Null => command.null(),
            };
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
