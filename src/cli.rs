use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
#[command(about, version)]
#[allow(clippy::struct_excessive_bools)]
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
    #[arg(long, conflicts_with = "command", required_unless_present = "command")]
    pub list: bool,

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

    /// Command to be executed in each folder
    #[arg(
        value_hint = ValueHint::CommandWithArguments,
        trailing_var_arg = true,
        conflicts_with = "list",
        required_unless_present = "list",
    )]
    pub command: Vec<OsString>,
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
