# `aoctool`: Advent of Code Downloader / Initializer

This tool contains the necessary scaffolding to download Advent of Code input files and create Rust
project templates.

Windows is not supported except via WSL.

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

Log in to the AoC site with whatever method you prefer. Then use the browser's dev tools to inspect
the cookies. You want the one called `session`. Configure this tool with it, so it can download the
inputs for you.

```bash
aoc config set --session "$SESSION"
```

### Annual Setup

If desired, initialize a top-level workspace in the current directory with

```bash
aoc init-year
```

That subcommand allows for inline configuration of templates, implementations, etc on a per-year
basis. To explore those options:

```bash
aoc init-year --help
```

The paths to the day's template files, to the implementation directory, and to the input files can
all be configured on an annual basis. For details, see

```bash
aoc config set --help
```

or edit the configuration directly with

```bash
open "$(aoc config path)"
```

## Per-day templating

```bash
aoc init
```

Within the configured year (if set) or in the current directory, this command will create a new
sub-crate and add it to the workspace, as well as downloading the problem's input. Inputs are saved
to a canonical directory. The sub-crate will be named for the day in question, so it can then be run
like

```bash
cargo run -p day01 -- --part2
```
