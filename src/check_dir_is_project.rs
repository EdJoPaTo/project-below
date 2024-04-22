use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

pub fn check_dir_is_project(patterns: &[Pattern], dir: &Path) -> bool {
    let mut state = patterns
        .iter()
        .map(|pattern| pattern.base.clone())
        .collect();
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
    let entries = dir
        .read_dir()?
        .filter_map(Result::ok)
        .map(|entry| entry.path());
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
        .filter(|pattern| state.contains(&pattern.base))
        .collect::<Vec<_>>();

    let all_still_possible = patterns
        .iter()
        .all(|pattern| pattern.position.can_descent());
    if !all_still_possible {
        return Ok(());
    }

    for dir in dirs {
        if let Some(name) = dir.file_name().and_then(std::ffi::OsStr::to_str) {
            let relevant_patterns = patterns
                .iter()
                .filter_map(|pattern| pattern.descent(name))
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
    raw: PathBuf,
}

#[derive(Debug, Clone)]
enum Position {
    Anywhere,
    Here,
    Below {
        direct: globset::GlobMatcher,
        below: Box<Self>,
    },
}

impl Position {
    fn new(position: &[&str]) -> Self {
        match position {
            [] => Self::Here,
            ["**"] => Self::Anywhere,
            ["**", ..] => panic!("glob pattern after ** is too deep"),
            [direct, below @ ..] => Self::Below {
                direct: globset::GlobBuilder::new(direct)
                    .literal_separator(true)
                    .build()
                    .expect("invalid glob pattern")
                    .compile_matcher(),
                below: Box::new(Self::new(below)),
            },
        }
    }

    const fn can_descent(&self) -> bool {
        !matches!(self, Self::Here)
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    base: Identifier,
    target: globset::GlobMatcher,
    position: Position,
}

impl Pattern {
    pub fn new_directory(pattern: PathBuf) -> Self {
        Self::new(pattern, Kind::Directory)
    }

    pub fn new_file(pattern: PathBuf) -> Self {
        Self::new(pattern, Kind::File)
    }

    fn new(raw: PathBuf, kind: Kind) -> Self {
        let splitted = raw
            .components()
            .filter(|component| !matches!(component, Component::Prefix(..) | Component::RootDir))
            .map(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .expect("pattern was already string, it should still be one")
            })
            .collect::<Vec<_>>();
        match splitted.as_slice() {
            [] => panic!("empty glob pattern"),
            [position @ .., target] => {
                let position = Position::new(position);
                let target = globset::GlobBuilder::new(target)
                    .literal_separator(true)
                    .build()
                    .expect("invalid glob pattern")
                    .compile_matcher();
                Self {
                    base: Identifier { kind, raw },
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
            Position::Below { direct, below } => direct.is_match(dir).then(|| Self {
                base: self.base.clone(),
                target: self.target.clone(),
                position: (**below).clone(),
            }),
        }
    }

    fn matches(&self, path: &Path) -> bool {
        if matches!(self.position, Position::Anywhere | Position::Here) {
            let kind_matches = match self.base.kind {
                Kind::File => path.is_file(),
                Kind::Directory => path.is_dir(),
            };
            if !kind_matches {
                return false;
            }
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map_or(false, |name| self.target.is_match(name))
        } else {
            false
        }
    }
}

#[test]
fn pattern_works_anywhere() {
    let kind = Kind::File;
    let raw: PathBuf = "**/*.rs".parse().unwrap();
    let result = Pattern::new(raw.clone(), kind);
    assert_eq!(result.base, Identifier { kind, raw });
    assert_eq!(result.target.glob().glob(), "*.rs");
    assert!(matches!(result.position, Position::Anywhere));
}

#[test]
fn pattern_works_in_base() {
    let kind = Kind::File;
    let raw: PathBuf = "*.rs".parse().unwrap();
    let result = Pattern::new(raw.clone(), kind);
    assert_eq!(result.base, Identifier { kind, raw });
    assert_eq!(result.target.glob().glob(), "*.rs");
    assert!(matches!(result.position, Position::Here));
}

#[test]
fn pattern_works_in_subdir() {
    let kind = Kind::File;
    let raw: PathBuf = "f*o/*.rs".parse().unwrap();
    let result = Pattern::new(raw.clone(), kind);
    assert_eq!(result.base, Identifier { kind, raw });
    assert_eq!(result.target.glob().glob(), "*.rs");
    if let Position::Below { direct, below } = result.position {
        assert_eq!(direct.glob().glob(), "f*o");
        assert!(matches!(*below, Position::Here));
    } else {
        panic!("wrong position");
    }
}

#[test]
fn pattern_works_anywhere_in_subdir() {
    let kind = Kind::File;
    let raw: PathBuf = "f*o/**/*.rs".parse().unwrap();
    let result = Pattern::new(raw.clone(), kind);
    assert_eq!(result.base, Identifier { kind, raw });
    assert_eq!(result.target.glob().glob(), "*.rs");
    if let Position::Below { direct, below } = result.position {
        assert_eq!(direct.glob().glob(), "f*o");
        assert!(matches!(*below, Position::Anywhere));
    } else {
        panic!("wrong position");
    }
}
