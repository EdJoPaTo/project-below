use clap::{app_from_crate, App, AppSettings, Arg, ValueHint};
use glob::Pattern;

#[must_use]
pub fn build() -> App<'static> {
    app_from_crate!()
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::new("directory")
                .short('d')
                .long("directory")
                .value_name("PATTERN")
                .value_hint(ValueHint::DirPath)
                .validator(Pattern::new)
                .multiple_occurrences(true)
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
                .validator(Pattern::new)
                .multiple_occurrences(true)
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
            Arg::new("command")
                .value_name("COMMAND")
                .value_hint(ValueHint::CommandWithArguments)
                .allow_invalid_utf8(true)
                .multiple_values(true)
                .required(true)
                .help("Command to be executed in each folder"),
        )
}

#[test]
fn verify_app() {
    build().debug_assert();
}
