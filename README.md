# `aoctool`: Advent of Code Downloader / Initializer

The AOC tool is an opinionated and configurable utility for handling the repetitive parts of
Advent of Code in Rust.

It's intended to synergize nicely with my [`aoclib` support library](https://github.com/coriolinus/aoclib/),
but there is no requirement that your implementation use that.

The actual workflow for day 1 of 2021 might look like this:

```bash
$ # Initialize a new workspace for 2021
$ aoc init-year --implementation adventofcode-2021 && cd adventofcode-2021
adventofcode-2021$ # Set the session key, acquired from the browser's cookies
adventofcode-2021$ aoc config set --session "$SESSION"
adventofcode-2021$ # Initialize a new sub-crate called day01 from templates, and download the input file.
adventofcode-2021$ aoc init
adventofcode-2021$ # Fill in the day01/src/lib.rs:part1() function body using the editor of your choice. Then:
adventofcode-2021$ cargo run -p day01
adventofcode-2021$ # Fill in the day01/src/lib.rs:part2() function body using the editor of your choice. Then:
adventofcode-2021$ cargo run -p day01 -- --part2 --no-part1
```

For each subsequent day, the only command required is `aoc init`.

## Installation

Windows is only supported via WSL.

```sh
cargo install --git "https://github.com/coriolinus/aoctool.git"
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

### Specifying the Templates

By default, each day's exercise will be initialized with the templates stored [here](https://github.com/coriolinus/aoctool/tree/master/day-template).
However, this behavior can be customized as desired. You can customize the directory where the templates are stored with

```bash
aoc config set --day-templates <path>
```

That path must be a directory containing three files: `Cargo.toml`, `src/lib.rs`, and `src/main.rs`. Those files can contain anything you like.
The following expressions are evaluated within the template: `{year}`, `{day}`, `{package_name}`.
