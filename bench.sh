#!/usr/bin/env bash
set -eu -o pipefail

BASE_DIR=$HOME/git
WARMUP=3

# Hint: fd is named fdfind in Debian repos (package: fd-find)

printf '\n#\n##\n### %s\n\n' '--directory=.git'
hyperfine --shell=none --warmup $WARMUP \
	"project-below --base-dir $BASE_DIR --directory=.git" \
	"fd --type=directory --prune --no-ignore-vcs --hidden '^\.git$' $BASE_DIR" \
	"fd --type=directory --prune --no-ignore-vcs --hidden --glob '.git' $BASE_DIR" \
	"find $BASE_DIR -name '.git' -type d"

printf '\n#\n##\n### %s\n\n' '--file=package.json'
hyperfine --shell=none --warmup $WARMUP \
	"project-below --base-dir $BASE_DIR --file=package.json" \
	"fd --type=file '^package\.json$' $BASE_DIR" \
	"fd --type=file --glob package.json $BASE_DIR" \
	"rg --files --glob=package.json $BASE_DIR" \
	"find $BASE_DIR -type f -name package.json"

printf '\n#\n##\n### %s\n\n' '--file=Cargo.toml'
hyperfine --shell=none --warmup $WARMUP \
	"project-below --base-dir $BASE_DIR --file=Cargo.toml" \
	"fd --type=file '^Cargo\.toml$' $BASE_DIR" \
	"fd --type=file --glob Cargo.toml $BASE_DIR" \
	"rg --files --glob=Cargo.toml $BASE_DIR" \
	"find $BASE_DIR -type f -name Cargo.toml"
