use aoclib::config::Config;
use aoctool::PathOpts;
use clap::{Args, Parser, Subcommand as DeriveSubcommand};
use color_eyre::eyre::{bail, Result};
use path_absolutize::Absolutize;
use time::OffsetDateTime as DateTime;

pub type Day = u8;
pub type Year = u32;

fn local() -> DateTime {
    DateTime::now_local().expect("local system has determinable local offset")
}

#[derive(Args, Clone, Copy, Debug)]
struct YearArg {
    /// Year (default: this year)
    #[arg(short, long)]
    year: Option<Year>,
}

impl YearArg {
    fn year(self) -> Year {
        self.year.unwrap_or_else(|| local().year() as Year)
    }
}

#[derive(Args, Clone, Copy, Debug)]
struct Date {
    /// Day (default: today's date)
    #[arg(short, long)]
    day: Option<Day>,

    #[command(flatten)]
    year: YearArg,
}

impl Date {
    fn day(self) -> Day {
        self.day.unwrap_or_else(|| local().day() as Day)
    }

    fn year(self) -> Year {
        self.year.year()
    }
}

#[derive(Parser, Debug)]
#[clap(about = "advent of code tool")]
enum Subcommand {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        cmd: ConfigOpts,
    },
    /// Emit the URL to a specified puzzle
    Url {
        #[command(flatten)]
        date: Date,
    },
    /// Initialize a puzzle
    Init {
        #[command(flatten)]
        date: Date,

        /// Do not create a sub-crate for the requested day
        #[arg(long)]
        skip_create_crate: bool,

        /// Do not attempt to fetch the input for the requested day
        #[arg(long)]
        skip_get_input: bool,
    },
    /// Initialize a repository for a year's solutions
    InitYear {
        #[command(flatten)]
        year: YearArg,
        #[command(flatten)]
        path_opts: PathOpts,
    },
    /// Clear templates.
    ClearTemplates {
        #[command(flatten)]
        year: YearArg,
    },
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
            Self::ClearTemplates { year } => {
                let config = Config::load().unwrap_or_default();
                aoctool::clear_templates(&config, year.year())?;
            }
        }
        Ok(())
    }
}

#[derive(DeriveSubcommand, Debug)]
enum ConfigOpts {
    /// Emit the path to the configuration file
    Path,
    /// Display the contents of the configuration file, if they exist
    Show,
    /// Set configuration
    Set {
        #[command(flatten)]
        year: YearArg,

        /// Website session key
        ///
        /// Log in to adventofcode.com and inspect the cookies to get this
        #[arg(short, long)]
        session: Option<String>,

        #[command(flatten)]
        path_opts: PathOpts,
    },
    /// Clear configuration
    Clear {
        #[command(flatten)]
        year: YearArg,

        /// Clear path to input files.
        #[arg(long)]
        input_files: bool,

        /// Clear path to this year's implementation directory.
        #[arg(long)]
        implementation: bool,

        /// Clear path to this year's day template files.
        #[arg(long)]
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
    let opt = Subcommand::parse();
    opt.run()
}
