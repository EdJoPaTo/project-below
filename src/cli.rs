use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::{Parser, ValueEnum, ValueHint};

#[derive(Debug, Parser)]
#[command(about, version)]
#[allow(clippy::partial_pub_fields, clippy::struct_excessive_bools)]
pub struct Cli {
    /// Base directory from where the search starts
    #[arg(
        long,
        value_name = "DIRECTORY",
        value_hint = ValueHint::DirPath,
        default_value = ".",
    )]
    pub base_dir: PathBuf,

    /// The project folder must contain a directory matching this glob pattern
    #[arg(
        short, long,
        value_name = "PATTERN",
        value_hint = ValueHint::DirPath,
        required_unless_present = "file",
    )]
    pub directory: Vec<PathBuf>,

    /// The project folder must contain a file matching this glob pattern
    #[arg(
        short, long,
        value_name = "PATTERN",
        value_hint = ValueHint::FilePath,
        required_unless_present = "directory",
    )]
    pub file: Vec<PathBuf>,

    /// Traverse into projects that already matched.
    ///
    /// This can be helpful for monorepos which include the configuration file in the main folder and each sub-folder.
    #[arg(long)]
    pub recursive: bool,

    /// Traverse into hidden folders to search for projects.
    #[arg(long)]
    pub hidden: bool,

    /// List all the directories instead of executing a command.
    ///
    /// This can be helpful for piping into other tools like `fzf`.
    ///
    /// This was previously required when no command was specified but can now safely omitted and will be removed in the next major release.
    #[arg(long, conflicts_with = "command")]
    list: bool,

    /// Separate listed paths by the null character.
    ///
    /// Useful for piping results. Does not work when executing a command.
    #[arg(long, conflicts_with = "command")]
    pub print0: bool,

    /// Print full, canonical paths. Shortcut for `--path-style=canonical`.
    #[arg(
        long,
        short,
        group = "path-output",
        help_heading = "Path Display Style"
    )]
    canonical: bool,

    /// Print paths relative to the current working directory.
    ///
    /// Use `--path-style=working-dir` instead. This argument will be removed in the next major release.
    #[arg(long, group = "path-output", help_heading = "Path Display Style")]
    relative: bool,

    /// Style to print the found project paths with.
    #[arg(
        long,
        value_enum,
        default_value_t = PathStyle::BaseDir,
        group = "path-output",
        help_heading = "Path Display Style"
    )]
    pub path_style: PathStyle,

    /// Minimal width for `--output=line-prefix` to occupy.
    ///
    /// This is useful for improved readability due to aligned output.
    #[arg(
        long,
        default_value_t = 40,
        requires = "command",
        help_heading = "Command Options"
    )]
    pub line_prefix_width: usize,

    /// Don't show the directory before the command.
    ///
    /// It's only shown on some `--output`. For example `inherit` only shows it when single-threaded.
    #[arg(long, requires = "command", help_heading = "Command Options")]
    pub no_header: bool,

    /// Define how the command output should be handled.
    #[arg(
        long,
        value_enum,
        default_value_t = CommandOutput::Inherit,
        requires = "command",
        help_heading = "Command Options"
    )]
    pub output: CommandOutput,

    /// Define whether a result of a finished command should be printed.
    ///
    /// The result includes the time it took, its exit code and the working directory.
    #[arg(
        long,
        value_enum,
        default_value_t = CommandResult::Always,
        requires = "command",
        help_heading = "Command Options"
    )]
    pub result: CommandResult,

    /// Execute multiple commands in parallel.
    ///
    /// A specific number of threads can be passed. Defaults to the available CPU cores.
    ///
    /// When this argument is not given commands are executed sequentially.
    /// Running multiple commands in parallel might not always be useful when they act on the same resource.
    /// For example `cargo fetch` uses the same cache where each process waits for the lock.
    #[arg(
        long,
        short = 'j',
        requires = "command",
        help_heading = "Command Options"
    )]
    #[allow(clippy::option_option)]
    threads: Option<Option<NonZeroUsize>>,

    /// Shortcut for `--no-header --result=never`.
    #[arg(
        long,
        conflicts_with_all = ["path-output", "no_header", "result"],
        group = "command-output-shortcut",
        requires = "command",
        help_heading = "Command Option Shortcuts"
    )]
    no_harness: bool,

    /// Shortcut for `--no-header --output=null`.
    #[arg(
        long,
        conflicts_with_all = ["no_header", "output"],
        group = "command-output-shortcut",
        requires = "command",
        help_heading = "Command Option Shortcuts"
    )]
    only_result: bool,

    /// Shortcut for `--no-header --output=null --result=never`.
    #[arg(
        long,
        short,
        conflicts_with_all = ["path-output", "no_header", "output", "result"],
        group = "command-output-shortcut",
        requires = "command",
        help_heading = "Command Option Shortcuts"
    )]
    quiet: bool,

    /// Command to be executed in each folder
    #[arg(
        value_hint = ValueHint::CommandWithArguments,
        trailing_var_arg = true,
        conflicts_with = "list",
    )]
    pub command: Vec<OsString>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PathStyle {
    /// Relative to the `--base-dir`
    BaseDir,
    /// Full, canonical path
    Canonical,
    /// Only the name of the directory without its path
    Dirname,
    /// Inspired by fish: `g/h/E/pu/project-below` is short for `git/hub/EdJoPaTo/public/project-below`
    Short,
    /// Relative to the current working directory instead of the `--base-dir`
    WorkingDir,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CommandOutput {
    /// Inherit stdout and stderr. When used with multiple threads it can create hard to understand output as everything mixes up.
    Inherit,
    /// Print the output prefixed per line with the directory of the process it outputted.
    LinePrefix,
    /// Collect the output and print it once the command finishes.
    Collect,
    /// Similar to attaching `/dev/null` to stdout / stderr.
    Null,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CommandResult {
    Always,
    Never,
    NonZero,
}
impl CommandResult {
    #[must_use]
    pub const fn print(self, success: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::NonZero => !success,
        }
    }
}

impl Cli {
    #[must_use]
    pub fn get() -> Self {
        let mut matches = Self::parse();
        if matches.list {
            eprintln!("project-below Hint: --list is no longer required and will be removed in the next major release");
        }
        if matches.canonical {
            matches.path_style = PathStyle::Canonical;
        }
        if matches.relative {
            eprintln!("project-below Hint: --relative is replaced by --path-style=working-dir and will be removed in the next major release");
            matches.path_style = PathStyle::WorkingDir;
        }
        if matches.no_harness {
            matches.no_header = true;
            matches.result = CommandResult::Never;
        }
        if matches.only_result {
            matches.no_header = true;
            matches.output = CommandOutput::Null;
        }
        if matches.quiet {
            matches.no_header = true;
            matches.output = CommandOutput::Null;
            matches.result = CommandResult::Never;
        }
        matches
    }

    #[must_use]
    pub fn threads(&self) -> NonZeroUsize {
        self.threads
            .and_then(|wanted| wanted.or_else(|| std::thread::available_parallelism().ok()))
            .unwrap_or(NonZeroUsize::MIN)
    }
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
