use std::fmt;
use std::path::{Path, PathBuf};

use crate::cli::PathStyle as CliPathStyle;
use crate::shortened_path::shortened_path;

pub enum PathStyle {
    Canonical,
    Dirname,
    BaseDir(PathBuf),
    Short(PathBuf),
    WorkingDir(PathBuf),
}

impl PathStyle {
    pub fn new(cli: CliPathStyle, base: PathBuf) -> Self {
        match cli {
            CliPathStyle::BaseDir => Self::BaseDir(base),
            CliPathStyle::Canonical => Self::Canonical,
            CliPathStyle::Dirname => Self::Dirname,
            CliPathStyle::Short => Self::Short(base),
            CliPathStyle::WorkingDir => {
                std::env::current_dir().map_or(Self::Canonical, Self::WorkingDir)
            }
        }
    }

    pub const fn path<'a>(&'a self, path: &'a Path) -> DPath<'a> {
        DPath { kind: self, path }
    }
}

pub struct DPath<'a> {
    path: &'a Path,
    kind: &'a PathStyle,
}
impl fmt::Display for DPath<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path;
        match &self.kind {
            PathStyle::BaseDir(base) => {
                let path = path.strip_prefix(base).unwrap_or(path);
                path.display().fmt(fmt)
            }
            PathStyle::Canonical => path
                .canonicalize()
                .as_deref()
                .unwrap_or(path)
                .display()
                .fmt(fmt),
            PathStyle::Dirname => {
                // Use path.filename.expect(…).display().fmt(fmt) once stabilized
                fmt.pad(
                    &path
                        .file_name()
                        .expect("Path should not be empty")
                        .to_string_lossy(),
                )
            }
            PathStyle::Short(base) => {
                if let Some(path) = shortened_path(path, base) {
                    fmt.pad(&path)
                } else {
                    path.display().fmt(fmt)
                }
            }
            PathStyle::WorkingDir(pwd) => {
                let relative = pathdiff::diff_paths(path, pwd);
                let path = relative.as_deref().unwrap_or(path);
                path.display().fmt(fmt)
            }
        }
    }
}
