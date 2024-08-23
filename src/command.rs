use std::ffi::OsString;
use std::path::Path;
use std::process::{Command as OsCommand, ExitStatus, Stdio};
use std::time::{Duration, Instant};

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
}
