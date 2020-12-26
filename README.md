# `aoctool`: Advent of Code Downloader / Initializer

This tool contains the necessary scaffolding to download Advent of Code input
files and create Rust project templates.

## Installation

### Simple

```sh
cargo install --git "https://github.com/coriolinus/aoctool.git"
```

### Your Fork

```sh
git clone "https://github.com/coriolinus/aoctool.git" aoctool
cd aoctool
# optional: edit template, etc
cargo install --path .
```

## Initial setup

Log in to the AoC site with whatever method you prefer. Then use the browser's dev tools to
inspect the cookies. You want the one called `session`. Configure this tool with it,
so it can download the inputs for you.

```bash
aoc config set --session "$SESSION"
```

### TODO:

If desired, initialize a top-level workspace in the current directory with

```bash
aoc year-init
```

### Other Options

If desired, you can specify a particular canonical path where input files
should be stored:

```bash
aoc config set --inputs "$DESIRED_PATH"
```

#### TODO:

set a config option from which the templates should be loaded

- if the specified location does not exist, it is created
- for each file in the default templates, if it does not exist in the dest
directory, it is copied from the default
- no file is ever clobbered

## Per-day templating

```bash
aoc init
```

Assuming the current directory is a cargo workspace, this command will  create
a new sub-crate and add it to the workspace, as well as downloading the
problem's input. Inputs are saved to a canonical directory. The sub-crate will
be named for the day in question, so it can then be run like

```bash
cargo run -p day01 -- --part2
```
