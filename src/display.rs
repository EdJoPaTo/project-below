use core::fmt;
use core::time::Duration;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::cli::{CommandResult, PathStyle as CliPathStyle};
use crate::shortened_path::shortened_path;

pub struct HarnessConfig {
    path_style: PathStyle,
    multithreaded: bool,
    line_prefix_width: usize,
    no_header: bool,
    result: CommandResult,

    need_linesplit: AtomicBool,
}

impl HarnessConfig {
    pub const fn new(
        path_style: PathStyle,
        threads: NonZeroUsize,
        line_prefix_width: usize,
        no_header: bool,
        result: CommandResult,
    ) -> Self {
        Self {
            path_style,
            multithreaded: threads.get() > 1,
            line_prefix_width,
            no_header,
            result,

            need_linesplit: AtomicBool::new(false),
        }
    }

    pub const fn create<'a>(&'a self, path: &'a Path) -> Harness<'a> {
        Harness { config: self, path }
    }
}

pub struct Harness<'a> {
    config: &'a HarnessConfig,
    path: &'a Path,
}

impl<'a> Harness<'a> {
    const fn path(&self) -> DPath<'a> {
        self.config.path_style.path(self.path)
    }

    pub fn inherit_header(&self) {
        if self.config.multithreaded || self.config.no_header {
            return;
        }
        let need_linesplit = self.config.need_linesplit.swap(true, Ordering::Relaxed);
        if need_linesplit {
            println!();
        }
        println!("{}", self.path());
    }

    pub fn line_prefix(&self) -> String {
        let width = self.config.line_prefix_width;
        format!("{:width$}  ", self.path())
    }

    pub fn result(&self, took: Duration, status: ExitStatus) {
        if self.config.result.print(status.success()) {
            let took = DDuration(took);
            println!("took {took}  {status} in {}", self.path());
        }
    }
}

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
