use std::collections::HashSet;
use std::path::{Component, Path};

pub fn check_dir_is_project(patterns: &[Pattern], dir: &Path) -> bool {
    let mut state = patterns.iter().map(|o| o.base.clone()).collect();
    drop(recursive(&mut state, dir, patterns));

    // When a pattern is matched successfully it will be removed.
    // If it is still there, it never matched.

    state.is_empty()
}

fn recursive(
    state: &mut HashSet<Identifier>,
    dir: &Path,
    patterns: &[Pattern],
) -> std::io::Result<()> {
    let entries = dir.read_dir()?.filter_map(Result::ok).map(|o| o.path());
    let mut dirs = Vec::new();

    for path in entries {
        let matched_patterns = patterns
            .iter()
            .filter(|pattern| pattern.matches(&path))
            .map(|pattern| &pattern.base);
        for to_be_removed in matched_patterns {
            state.remove(to_be_removed);
        }

        if path.is_dir() {
            dirs.push(path);
        }
    }

    let patterns = patterns
        .iter()
        .filter(|o| state.contains(&o.base))
        .collect::<Vec<_>>();

    let all_still_possible = patterns.iter().all(|o| o.position.can_descent());
    if !all_still_possible {
        return Ok(());
    }

    for dir in dirs {
        if let Some(name) = dir.file_name().and_then(std::ffi::OsStr::to_str) {
            let relevant_patterns = patterns
                .iter()
                .filter_map(|o| o.descent(name))
                .collect::<Vec<_>>();

            if !relevant_patterns.is_empty() {
                drop(recursive(state, &dir, &relevant_patterns));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
enum Kind {
    File,
    Directory,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Identifier {
    kind: Kind,
    raw: String,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
enum Position {
    Anywhere,
    Here,
    Below {
        direct: glob::Pattern,
        below: Box<Self>,
    },
}

impl Position {
    fn new<'i, I>(position: I) -> Self
    where
        I: IntoIterator<Item = &'i str>,
    {
        let position = position.into_iter().collect::<Vec<_>>();
        match position.as_slice() {
            [] => Self::Here,
            ["**"] => Self::Anywhere,
            ["**", ..] => panic!("glob pattern after ** is too deep"),
            [direct, below @ ..] => {
                let below = Self::new(below.iter().map(std::ops::Deref::deref));
                Self::Below {
                    direct: glob::Pattern::new(direct).expect("invalid glob pattern"),
                    below: Box::new(below),
                }
            }
        }
    }

    const fn can_descent(&self) -> bool {
        match self {
            Position::Here => false,
            Position::Anywhere | Position::Below { .. } => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    base: Identifier,
    target: glob::Pattern,
    position: Position,
}

impl Pattern {
    pub fn new_directory(p: &str) -> Self {
        Self::new(p, Kind::Directory)
    }

    pub fn new_file(p: &str) -> Self {
        Self::new(p, Kind::File)
    }

    fn new(p: &str, kind: Kind) -> Self {
        let splitted = Path::new(p)
            .components()
            .filter(|o| !matches!(o, Component::Prefix(..) | Component::RootDir))
            .map(|o| {
                o.as_os_str()
                    .to_str()
                    .expect("pattern was already string, it should still be one")
            })
            .collect::<Vec<_>>();

        match splitted.as_slice() {
            [] => panic!("empty glob pattern"),
            [position @ .., target] => {
                let position = Position::new(position.iter().map(std::ops::Deref::deref));
                let target = glob::Pattern::new(target).expect("invalid glob pattern");
                Self {
                    base: Identifier {
                        raw: p.to_string(),
                        kind,
                    },
                    target,
                    position,
                }
            }
        }
    }

    fn descent(&self, dir: &str) -> Option<Self> {
        match &self.position {
            Position::Anywhere => Some(self.clone()),
            Position::Here => None,
            Position::Below { direct, below } => {
                if direct.matches(dir) {
                    Some(Self {
                        base: self.base.clone(),
                        target: self.target.clone(),
                        position: (**below).clone(),
                    })
                } else {
                    None
                }
            }
        }
    }

    fn matches(&self, path: &Path) -> bool {
        let kind_matches = match self.base.kind {
            Kind::File => path.is_file(),
            Kind::Directory => path.is_dir(),
        };
        if !kind_matches {
            return false;
        }

        if matches!(self.position, Position::Anywhere | Position::Here) {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map_or(false, |name| self.target.matches(name))
        } else {
            false
        }
    }
}

#[test]
fn pattern_works_anywhere() {
    let kind = Kind::File;
    let raw = "**/*.rs".to_string();
    assert_eq!(
        Pattern::new(&raw, kind),
        Pattern {
            base: Identifier { kind, raw },
            target: glob::Pattern::new("*.rs").unwrap(),
            position: Position::Anywhere,
        }
    );
}

#[test]
fn pattern_works_in_base() {
    let kind = Kind::File;
    let raw = "*.rs".to_string();
    assert_eq!(
        Pattern::new(&raw, kind),
        Pattern {
            base: Identifier { kind, raw },
            target: glob::Pattern::new("*.rs").unwrap(),
            position: Position::Here,
        }
    );
}

#[test]
fn pattern_works_in_subdir() {
    let kind = Kind::File;
    let raw = "f*o/*.rs".to_string();
    assert_eq!(
        Pattern::new(&raw, kind),
        Pattern {
            base: Identifier { kind, raw },
            target: glob::Pattern::new("*.rs").unwrap(),
            position: Position::Below {
                direct: glob::Pattern::new("f*o").unwrap(),
                below: Box::new(Position::Here),
            },
        }
    );
}

#[test]
fn pattern_works_anywhere_in_subdir() {
    let kind = Kind::File;
    let raw = "f*o/**/*.rs".to_string();
    assert_eq!(
        Pattern::new(&raw, kind),
        Pattern {
            base: Identifier { kind, raw },
            target: glob::Pattern::new("*.rs").unwrap(),
            position: Position::Below {
                direct: glob::Pattern::new("f*o").unwrap(),
                below: Box::new(Position::Anywhere),
            },
        }
    );
}
