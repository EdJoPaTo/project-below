use core::fmt;
use core::time::Duration;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use crate::cli::PathStyle as CliPathStyle;

pub enum PathStyle {
    Canonical,
    Dirname,
    BaseDir(PathBuf),
    WorkingDir(PathBuf),
}

impl PathStyle {
    pub fn new(cli: CliPathStyle, base: PathBuf) -> Self {
        match cli {
            CliPathStyle::BaseDir => Self::BaseDir(base),
            CliPathStyle::Canonical => Self::Canonical,
            CliPathStyle::Dirname => Self::Dirname,
            CliPathStyle::WorkingDir => {
                std::env::current_dir().map_or(Self::Canonical, Self::WorkingDir)
            }
        }
    }

    pub const fn path<'a>(&'a self, path: &'a Path) -> DPath<'a> {
        DPath { kind: self, path }
    }

    pub fn print_endline(&self, path: &Path, took: Duration, status: ExitStatus) {
        let took = DDuration(took);
        println!("took {took}  {status} in {}", self.path(path));
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
            PathStyle::Canonical => path.display().fmt(fmt),
            PathStyle::Dirname => {
                // Use path.filename.expect(…).display().fmt(fmt) once stabilized
                fmt.pad(
                    &path
                        .file_name()
                        .expect("Path should not be empty")
                        .to_string_lossy(),
                )
            }
            PathStyle::WorkingDir(pwd) => {
                let relative = pathdiff::diff_paths(path, pwd);
                let path = relative.as_deref().unwrap_or(path);
                path.display().fmt(fmt)
            }
        }
    }
}

struct DDuration(Duration);
impl fmt::Display for DDuration {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.0.as_secs() % 60;
        let minutes = (self.0.as_secs() / 60) % 60;
        let hours = self.0.as_secs() / (60 * 60);

        if hours > 0 {
            write!(fmt, "{hours:>3}h")?;
        } else {
            fmt.write_str("    ")?;
        }
        if minutes > 0 {
            write!(fmt, "{minutes:>2}m")?;
        } else {
            fmt.write_str("   ")?;
        }
        if seconds > 0 {
            write!(fmt, "{seconds:>2}s")?;
        } else {
            fmt.write_str("   ")?;
        }
        if hours == 0 && minutes == 0 {
            write!(fmt, "{:>3}ms", self.0.subsec_millis())
        } else {
            fmt.write_str("     ")
        }
    }
}

#[cfg(test)]
mod duration_tests {
    use super::*;

    #[track_caller]
    fn case(expected: &str, duration: Duration) {
        let actual = format!("{}", DDuration(duration));
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 15);
    }

    #[test]
    fn few_nanos() {
        case("            0ms", Duration::from_nanos(42));
    }

    #[test]
    fn few_ms() {
        case("           42ms", Duration::from_millis(42));
    }

    #[test]
    fn leet() {
        case("        1s337ms", Duration::from_millis(1337));
    }

    #[test]
    fn few_minutes() {
        case("     3m12s     ", Duration::from_millis(192_042));
    }

    #[test]
    fn many_minutes() {
        case("    14m52s     ", Duration::from_millis(892_042));
    }

    #[test]
    fn some_hours() {
        case("  2h46m40s     ", Duration::from_millis(10_000_042));
    }

    #[test]
    fn some_days() {
        case("138h53m20s     ", Duration::from_millis(500_000_042));
    }

    #[test]
    fn exact_hours_and_seconds() {
        case("  5h   12s     ", Duration::from_secs(5 * 60 * 60 + 12));
    }
}
