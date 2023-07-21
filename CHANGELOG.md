# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Print paths canonical with `--canonical`
- Traverse into hidden folders to search for projects with `--hidden`

### Changed

- Use `OsString` for `--file` and `--directory`. This allows characters that are possible for the OS but aren't Unicode.

## [1.0.0] - 2023-04-28

This tool is used daily for over a year now. I consider this stable. (no breaking change from v0.2)

### Changed

- Do less memory allocations when doing stdout
- Improve error message when command failed to execute

## [0.2.1] - 2022-04-15

### Added

- Build deb/rpm packages

## [0.2.0] - 2022-02-14

### Added

- `--list` shows a list of all the matching directories instead of running a command.
- Set the `--base-dir` from where to search for directories.
- Check deep globs like `src/*.ts`.

### Changed

- Speed things up with a parallel file walker.
- Performance improvements.
- Skip the base directory for project checks. Only scan subdirectories as potential projects.

## [0.1.0] - 2022-02-03

Initial release
