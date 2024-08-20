#!/usr/bin/env bash
set -eu -o pipefail

BASE_DIR=$HOME/git
WARMUP=3

# Hint: fd is named fdfind in Debian repos (package: fd-find)

hyperfine --shell=none --warmup $WARMUP \
	"find $BASE_DIR -name '.git' -type d" \
	"fd --threads=1 --type=directory --prune --no-ignore-vcs --hidden '^\.git$' $BASE_DIR" \
	"fd --threads=1 --type=directory --prune --no-ignore-vcs --hidden --glob '.git' $BASE_DIR" \
	"project-below --base-dir $BASE_DIR --directory=.git"

hyperfine --shell=none --warmup $WARMUP \
	"find $BASE_DIR -type f -name package.json" \
	"fd --threads=1 --type=file '^package\.json$' $BASE_DIR" \
	"fd --threads=1 --type=file --glob package.json $BASE_DIR" \
	"rg --threads=1 --files --glob=package.json $BASE_DIR" \
	"project-below --base-dir $BASE_DIR --file=package.json"

hyperfine --shell=none --warmup $WARMUP \
	"find $BASE_DIR -type f -name Cargo.toml" \
	"fd --threads=1 --type=file '^Cargo\.toml$' $BASE_DIR" \
	"fd --threads=1 --type=file --glob Cargo.toml $BASE_DIR" \
	"rg --threads=1 --files --glob=Cargo.toml $BASE_DIR" \
	"project-below --base-dir $BASE_DIR --file=Cargo.toml"
