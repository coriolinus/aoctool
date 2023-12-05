use aoclib::\{config::Config, website::get_input};
use {package_name}::\{part1, part2};

use clap::Parser;
use color_eyre::eyre::Result;
use std::path::PathBuf;

const YEAR: u32 = {year};
const DAY: u8 = {day};

#[derive(Parser, Debug)]
struct RunArgs \{
    /// input file
    ///
    /// when unset, defaults to `inputs/input-NN.txt`, where `NN` is the current day
    input: Option<PathBuf>,

    /// skip part 1
    #[arg(long)]
    no_part1: bool,

    /// run part 2
    #[arg(long)]
    part2: bool,
}

impl RunArgs \{
    fn input(&self) -> Result<PathBuf> \{
        match self.input \{
            None => \{
                let config = Config::load()?;
                // this does nothing if the input file already exists, but
                // simplifies the workflow after cloning the repo on a new computer
                get_input(&config, YEAR, DAY)?;
                Ok(config.input_for(YEAR, DAY))
            }
            Some(ref path) => Ok(path.clone()),
        }
    }
}

fn main() -> Result<()> \{
    color_eyre::install()?;
    let args = RunArgs::parse();
    let input_path = args.input()?;

    if !args.no_part1 \{
        part1(&input_path)?;
    }
    if args.part2 \{
        part2(&input_path)?;
    }
    Ok(())
}
