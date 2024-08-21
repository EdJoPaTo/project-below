use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use ignore::WalkBuilder;

use crate::check_dir_is_project::{check_dir_is_project, Pattern};

pub fn walk(
    base_dir: &Path,
    patterns: Vec<Pattern>,
    include_hidden: bool,
    recursive: bool,
) -> Receiver<PathBuf> {
    let (tx, rx) = channel();
    let walk = WalkBuilder::new(base_dir)
        .hidden(!include_hidden)
        .filter_entry(|dir_entry| {
            dir_entry
                .file_type()
                .map_or(false, |file_type| file_type.is_dir())
        })
        .build_parallel();
    thread::spawn(move || {
        walk.run(|| {
            let patterns = patterns.clone();
            let tx = tx.clone();
            Box::new(move |entry| {
                match entry {
                    Ok(dir_entry) => {
                        if let Some(err) = dir_entry.error() {
                            eprintln!("Warning for path {:?}: {err}", dir_entry.path());
                        }
                        if dir_entry.depth() == 0 {
                            return ignore::WalkState::Continue;
                        }
                        let path = dir_entry.into_path();
                        if check_dir_is_project(&patterns, &path) {
                            if tx.send(path).is_err() {
                                return ignore::WalkState::Quit;
                            }
                            if !recursive {
                                return ignore::WalkState::Skip;
                            }
                        }
                    }
                    Err(err) => eprintln!("Couldn't enter directory {err}"),
                }
                ignore::WalkState::Continue
            })
        });
    });
    rx
}
