use std::ffi::OsString;
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
    #[deprecated = "Omit or command is completely fine"]
    list: bool,

    /// Separate listed paths by the null character.
    ///
    /// Useful for piping results. Does not work when executing a command.
    #[arg(long, conflicts_with = "command")]
    pub print0: bool,

    /// Print canonical paths.
    ///
    /// This shows the full path instead of relative to the base-dir.
    #[arg(long, group = "path-output")]
    pub canonical: bool,

    /// Print paths relative to the current working directory.
    ///
    /// This shows the path relative to the working directory instead of relative to the base-dir.
    #[arg(long, group = "path-output")]
    pub relative: bool,

    /// Shortcut for `--no-header --result=never`.
    #[arg(
        long,
        conflicts_with_all = ["path-output", "no_header", "result"],
        requires = "command",
        help_heading = "Command Options"
    )]
    no_harness: bool,

    /// Don't show the directory before the command.
    #[arg(long, requires = "command", help_heading = "Command Options")]
    pub no_header: bool,

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

    /// Command to be executed in each folder
    #[arg(
        value_hint = ValueHint::CommandWithArguments,
        trailing_var_arg = true,
        conflicts_with = "list",
    )]
    pub command: Vec<OsString>,
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
        #[allow(deprecated)]
        if matches.list {
            eprintln!("project-below Hint: --list is no longer required and will be removed in the next major release");
        }
        if matches.no_harness {
            matches.no_header = true;
            matches.result = CommandResult::Never;
        }
        matches
    }
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
