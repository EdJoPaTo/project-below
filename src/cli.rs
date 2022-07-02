use clap::builder::ValueParser;
use clap::{command, Arg, Command, ValueHint};

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn build() -> Command<'static> {
    command!()
        .trailing_var_arg(true)
        .arg(
            Arg::new("base")
                .long("base-dir")
                .value_name("DIR")
                .value_hint(ValueHint::DirPath)
                .value_parser(ValueParser::path_buf())
                .takes_value(true)
                .default_value(".")
                .help("Base directory from where the search starts"),
        )
        .arg(
            Arg::new("directory")
                .short('d')
                .long("directory")
                .value_name("PATTERN")
                .value_hint(ValueHint::DirPath)
                .action(clap::ArgAction::Append)
                .takes_value(true)
                .required_unless_present_any(&["file"])
                .help("The project folder must contain a directory matching this glob pattern"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("PATTERN")
                .value_hint(ValueHint::FilePath)
                .action(clap::ArgAction::Append)
                .takes_value(true)
                .required_unless_present_any(&["directory"])
                .help("The project folder must contain a file matching this glob pattern"),
        )
        .arg(
            Arg::new("recursive")
                .long("recursive")
                .help("Traverse into projects that already matched")
                .long_help("Traverse into projects that already matched. This can be helpful for monorepos which include the config file in the main folder and each subfolder."),
        )
        .arg(
            Arg::new("list")
                .long("list")
                .help("List all the directories instead of executing a command")
                .long_help("List all the directories instead of executing a command. This can be helpful for piping into other tools like fzf."),
        )
        .arg(
            Arg::new("command")
                .value_name("COMMAND")
                .value_hint(ValueHint::CommandWithArguments)
                .value_parser(ValueParser::os_string())
                .multiple_values(true)
                .conflicts_with("list")
                .required_unless_present("list")
                .help("Command to be executed in each folder"),
        )
}

#[test]
fn verify() {
    build().debug_assert();
}
