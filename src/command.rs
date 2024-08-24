use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::process::{Command as OsCommand, ExitStatus, Stdio};
use std::thread::Scope;
use std::time::{Duration, Instant};

use crate::byte_lines::ReadByteLines;

pub struct Command(OsCommand);

impl Command {
    pub fn new(raw: &[OsString], working_dir: &Path) -> Self {
        let mut command = OsCommand::new(&raw[0]);
        command.args(&raw[1..]).current_dir(working_dir);
        Self(command)
    }

    #[track_caller]
    fn failed(&self, err: &std::io::Error) -> ! {
        let program = self.0.get_program();
        panic!("failed to execute process {program:?}: {err}");
    }

    pub fn inherit(mut self) -> (ExitStatus, Duration) {
        let start = Instant::now();
        let status = self
            .0
            .env("PAGER", "cat")
            .status()
            .unwrap_or_else(|err| self.failed(&err));
        let took = start.elapsed();
        (status, took)
    }

    pub fn null(mut self) -> (ExitStatus, Duration) {
        let start = Instant::now();
        let status = self
            .0
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap_or_else(|err| self.failed(&err));
        let took = start.elapsed();
        (status, took)
    }

    pub fn lineprefixed(mut self, prefix: &str) -> (ExitStatus, Duration) {
        let start = Instant::now();
        let mut child = self
            .0
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|err| self.failed(&err));

        std::thread::scope(|scope| {
            scope_spawn(scope, "command-stdout", || {
                for line in child.stdout.take().unwrap().byte_lines() {
                    let mut stdout = std::io::stdout().lock();
                    stdout.write_all(prefix.as_bytes()).unwrap();
                    stdout.write_all(&line).unwrap();
                    stdout.write_all(b"\n").unwrap();
                }
            });
            scope_spawn(scope, "command-stderr", || {
                for line in child.stderr.take().unwrap().byte_lines() {
                    let mut stderr = std::io::stderr().lock();
                    stderr.write_all(prefix.as_bytes()).unwrap();
                    stderr.write_all(&line).unwrap();
                    stderr.write_all(b"\n").unwrap();
                }
            });
        });

        let output = child
            .wait_with_output()
            .expect("failed to wait on child process");
        let took = start.elapsed();

        assert!(output.stdout.is_empty(), "stdout should be empty");
        assert!(output.stderr.is_empty(), "stderr should be empty");

        (output.status, took)
    }
}

fn scope_spawn<'scope, F>(scope: &'scope Scope<'scope, '_>, name: &str, func: F)
where
    F: FnOnce() + Send + 'scope,
{
    std::thread::Builder::new()
        .name(name.to_owned())
        .spawn_scoped(scope, func)
        .expect("failed to spawn thread");
}
