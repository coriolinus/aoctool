use aoclib::config::Config;
use aoctool::PathOpts;
use chrono::{Datelike, Local};
use color_eyre::eyre::{bail, Result};
use path_absolutize::Absolutize;
use structopt::StructOpt;

#[derive(StructOpt, Clone, Copy, Debug)]
struct Year {
    /// Year (default: this year)
    #[structopt(short, long)]
    year: Option<u32>,
}

impl Year {
    fn year(self) -> u32 {
        self.year.unwrap_or_else(|| Local::now().year() as u32)
    }
}

#[derive(StructOpt, Clone, Copy, Debug)]
struct Date {
    /// Day (default: today's date)
    #[structopt(short, long)]
    day: Option<u8>,

    #[structopt(flatten)]
    year: Year,
}

impl Date {
    fn day(self) -> u8 {
        self.day.unwrap_or_else(|| Local::now().day() as u8)
    }

    fn year(self) -> u32 {
        self.year.year()
    }
}

#[derive(StructOpt, Debug)]
#[structopt(about = "advent of code tool")]
enum Subcommand {
    /// Manage configuration
    Config {
        #[structopt(subcommand)]
        cmd: ConfigOpts,
    },
    /// Emit the URL to a specified puzzle
    Url {
        #[structopt(flatten)]
        date: Date,
    },
    /// Initialize a puzzle
    Init {
        #[structopt(flatten)]
        date: Date,

        /// Do not create a sub-crate for the requested day
        #[structopt(long)]
        skip_create_crate: bool,

        /// Do not attempt to fetch the input for the requested day
        #[structopt(long)]
        skip_get_input: bool,
    },
    /// Initialize a repository for a year's solutions
    InitYear {
        #[structopt(flatten)]
        year: Year,
        #[structopt(flatten)]
        path_opts: PathOpts,
    },
    /// Clear templates.
    ClearTemplates {
        #[structopt(flatten)]
        year: Year,
    }
}

impl Subcommand {
    fn run(self) -> Result<()> {
        match self {
            Self::Config { cmd } => cmd.run()?,
            Self::Url { date } => {
                println!("{}", aoclib::website::url_for_day(date.year(), date.day()));
            }
            Self::Init {
                date,
                skip_create_crate,
                skip_get_input,
            } => {
                let config = Config::load()?;
                aoctool::initialize(
                    &config,
                    date.year(),
                    date.day(),
                    skip_create_crate,
                    skip_get_input,
                )?;
            }
            Self::InitYear { year, path_opts } => {
                let mut config = Config::load().unwrap_or_default();
                aoctool::initialize_year(&mut config, year.year(), path_opts)?;
                config.save()?;
            }
            Self::ClearTemplates {
                year,
            } => {
                let config = Config::load().unwrap_or_default();
                aoctool::clear_templates(&config, year.year())?;
            }
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
enum ConfigOpts {
    /// Emit the path to the configuration file
    Path,
    /// Display the contents of the configuration file, if they exist
    Show,
    /// Set configuration
    Set {
        #[structopt(flatten)]
        year: Year,

        /// Website session key
        ///
        /// Log in to adventofcode.com and inspect the cookies to get this
        #[structopt(short, long)]
        session: Option<String>,

        #[structopt(flatten)]
        path_opts: PathOpts,
    },
    /// Clear configuration
    Clear {
        #[structopt(flatten)]
        year: Year,

        /// Clear path to input files.
        #[structopt(long)]
        input_files: bool,

        /// Clear path to this year's implementation directory.
        #[structopt(long)]
        implementation: bool,

        /// Clear path to this year's day template files.
        #[structopt(long)]
        day_template: bool,
    },
}

impl ConfigOpts {
    fn run(self) -> Result<()> {
        match self {
            Self::Path => println!("{}", aoclib::config::path().display()),
            Self::Show => {
                let data = std::fs::read_to_string(aoclib::config::path())?;
                println!("{}", data);
            }
            Self::Set {
                year,
                session,
                path_opts:
                    PathOpts {
                        input_files,
                        implementation,
                        day_templates,
                    },
            } => {
                let mut config = Config::load().unwrap_or_default();
                if let Some(session) = session {
                    if session.is_empty() {
                        bail!("session key must not be empty");
                    }
                    config.session = session;
                }
                if let Some(path) = input_files {
                    if path.exists() && !path.is_dir() {
                        bail!("input_files must be a directory");
                    }
                    config.set_input_files(year.year(), path.absolutize()?.into_owned());
                }
                if let Some(path) = implementation {
                    if path.exists() && !path.is_dir() {
                        bail!("implementation must be a directory");
                    }
                    config.set_implementation(year.year(), path.absolutize()?.into_owned());
                }
                if let Some(path) = day_templates {
                    if path.exists() && !path.is_dir() {
                        bail!("day-templates must be a directory");
                    }
                    config.set_day_template(year.year(), path.absolutize()?.into_owned());
                }
                config.save()?;
            }
            Self::Clear {
                year,
                input_files,
                implementation,
                day_template,
            } => {
                let mut config = Config::load().unwrap_or_default();
                let mut paths = config.paths.entry(year.year()).or_default();
                if input_files {
                    paths.input_files = None;
                }
                if implementation {
                    paths.implementation = None;
                }
                if day_template {
                    paths.day_template = None;
                }
                config.save()?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let opt = Subcommand::from_args();
    opt.run()
}
