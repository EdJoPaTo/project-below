use std::io::Write;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::{ExitStatus, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::cli::CommandResult;
use crate::path_style::{DPath, PathStyle};

pub struct Config {
    path_style: PathStyle,
    multithreaded: bool,
    line_prefix_width: usize,
    no_header: bool,
    result: CommandResult,

    first: AtomicBool,
    need_linesplit: AtomicBool,
}

impl Config {
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

            first: AtomicBool::new(true),
            need_linesplit: AtomicBool::new(false),
        }
    }

    pub const fn create<'a>(&'a self, path: &'a Path) -> Harness<'a> {
        Harness { config: self, path }
    }
}

pub struct Harness<'a> {
    config: &'a Config,
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
        let first = self.config.first.swap(false, Ordering::Relaxed);
        if !first {
            println!();
        }
        println!("{}", self.path());
    }

    pub fn line_prefix(&self) -> String {
        let width = self.config.line_prefix_width;
        format!("{:width$}  ", self.path())
    }

    pub fn collect(&self, output: &Output) {
        let has_output = !output.stdout.is_empty() || !output.stderr.is_empty();
        let show_result = self.config.result.print(output.status.success());
        if !has_output && !show_result {
            return;
        }
        let has_any_harness =
            !self.config.no_header || !matches!(self.config.result, CommandResult::Never);
        if has_any_harness {
            let first = self.config.first.swap(false, Ordering::Relaxed);
            let need_linesplit = self
                .config
                .need_linesplit
                .swap(has_output, Ordering::Relaxed);
            if need_linesplit || (has_output && !first) {
                println!();
            }
        }
        if let Some(last) = output.stdout.last() {
            if !self.config.no_header {
                println!("{}  stdout:", self.path());
            }
            std::io::stdout().write_all(&output.stdout).unwrap();
            if *last != b'\n' {
                std::io::stdout().write_all(b"\n").unwrap();
            }
        }
        if let Some(last) = output.stderr.last() {
            if !self.config.no_header {
                eprintln!("{}  stderr:", self.path());
            }
            std::io::stderr().write_all(&output.stderr).unwrap();
            if *last != b'\n' {
                std::io::stderr().write_all(b"\n").unwrap();
            }
        }
    }

    pub fn result(&self, took: Duration, status: ExitStatus) {
        if self.config.result.print(status.success()) {
            let took = crate::took::Took(took);
            println!("took {took}  {status} in {}", self.path());
        }
    }
}
