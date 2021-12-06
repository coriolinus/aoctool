use aoclib::parse;
use std::path::Path;

pub fn part1(input: &Path) -> Result<(), Error> \{
    unimplemented!("input file: \{:?}", input)
}

pub fn part2(input: &Path) -> Result<(), Error> \{
    unimplemented!("input file: \{:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error \{
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
