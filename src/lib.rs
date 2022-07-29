use path_absolutize::Absolutize;
use serde::Serialize;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;
use thiserror::Error;
use tinytemplate::TinyTemplate;
use toml_edit::Document;

use aoclib::config::Config;

const TEMPLATE_FILES: &[&str] = &["Cargo.toml", "src/lib.rs", "src/main.rs"];

/// Get `Cargo.toml` of the implementation directory.
///
/// Return its path and the parsed `Document`.
fn get_cargo_toml(config: &Config, year: u32) -> Result<(PathBuf, Document), Error> {
    // parse the local Cargo.toml to discover if we're in the right place
    let cargo_toml_path = config.implementation(year).join("Cargo.toml");
    if !cargo_toml_path.exists() {
        Err(Error::NoCargoToml)?;
    }
    let manifest = Document::from_str(
        &std::fs::read_to_string(&cargo_toml_path)
            .map_err(|err| Error::Io("reading Cargo.toml", err))?,
    )?;

    Ok((cargo_toml_path, manifest))
}

fn add_crate_to_workspace(
    cargo_toml_path: &Path,
    manifest: &mut Document,
    crate_name: &str,
) -> Result<(), Error> {
    use toml_edit::{Array, Item, Table, Value};

    let root_table = manifest.as_table_mut();

    let workspace = root_table
        .entry("workspace")
        .or_insert(Item::Table(Table::new()));
    let workspace = workspace.as_table_mut().ok_or(Error::MalformedToml)?;

    let members = workspace
        .entry("members")
        .or_insert(Item::Value(Value::Array(Array::new())));
    let members = members
        .as_value_mut()
        .ok_or(Error::MalformedToml)?
        .as_array_mut()
        .ok_or(Error::MalformedToml)?;

    if members.iter().any(|item| {
        item.as_str()
            .map(|item_str| item_str == crate_name)
            .unwrap_or_default()
    }) {
        Err(Error::CrateAlreadyExists(crate_name.to_string()))?;
    }

    members.push(crate_name);

    std::fs::write(cargo_toml_path, manifest.to_string())
        .map_err(|err| Error::Io("writing updated Cargo.toml", err))?;
    Ok(())
}

/// Ensure the template directory from the configuration exists and is initialized.
fn ensure_template_dir(config: &Config, year: u32) -> Result<PathBuf, Error> {
    let template_dir = config.day_template(year);
    for template in TEMPLATE_FILES {
        let template_path = template_dir.join(template);
        if !template_path.exists() {
            // if we have a subdirectory of template_dir, like `crate/src/foo.rs`, this will ensure everything exists
            if let Some(parent) = template_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|err| Error::Io("creating template parent directory", err))?;
            }
            let url = format!(
                "https://raw.githubusercontent.com/coriolinus/aoctool/master/day-template/{}",
                template
            );
            let client = reqwest::blocking::Client::builder()
                .gzip(true)
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(Error::ClientBuilder)?;
            let mut response = client
                .get(&url)
                .send()
                .map_err(Error::RequestingInput)?
                .error_for_status()
                .map_err(Error::ResponseStatus)?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(template_path)
                .map_err(|err| Error::Io("creating template file", err))?;
            response.copy_to(&mut file).map_err(Error::Downloading)?;
        }
    }
    Ok(template_dir)
}

fn render_templates_into(
    config: &Config,
    day_dir: &Path,
    year: u32,
    day: u8,
    day_name: &str,
) -> Result<(), Error> {
    #[derive(Serialize)]
    struct Context {
        year: u32,
        day: u8,
        package_name: String,
    }

    let context = Context {
        year,
        day,
        package_name: day_name.to_string(),
    };

    // render templates
    let template_dir = ensure_template_dir(config, year)?;
    for template in TEMPLATE_FILES {
        let mut tt = TinyTemplate::new();
        let template_text = std::fs::read_to_string(template_dir.join(template))
            .map_err(|err| Error::Io("reading template file", err))?;
        tt.add_template(template, &template_text)
            .map_err(|err| Error::Template(err, template.to_string()))?;
        let rendered_text = tt
            .render(template, &context)
            .map_err(|err| Error::Template(err, template.to_string()))?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(day_dir.join(template))
            .map_err(|err| Error::Io("opening template destination for writing", err))?;
        file.write_all(rendered_text.as_bytes())
            .map_err(|err| Error::Io("writing rendered template", err))?;
    }

    Ok(())
}

/// Initialize a new day.
///
/// This entails:
///
/// - ensuring we're in the right crate
/// - creating a new sub-crate
/// - updating the workspaces of this crate
/// - copying in a few templates to set up the day
/// - downloading the puzzle input
pub fn initialize(
    config: &Config,
    year: u32,
    day: u8,
    skip_create_crate: bool,
    skip_get_input: bool,
) -> Result<(), Error> {
    let implementation_dir = config.implementation(year);
    let (cargo_toml_path, mut manifest) = get_cargo_toml(config, year)?;

    if !skip_create_crate {
        // set up new sub-crate basics
        let day_name = format!("day{:02}", day);
        let day_dir = implementation_dir.join(&day_name);
        std::fs::create_dir_all(day_dir.join("src"))
            .map_err(|err| Error::Io("creating day dir", err))?;

        // update the workspaces of this crate
        add_crate_to_workspace(&cargo_toml_path, &mut manifest, &day_name)?;

        // render templates, creating new sub-crate
        render_templates_into(config, &day_dir, year, day, &day_name)?;
    }

    if !skip_get_input {
        // download the input
        aoclib::website::get_input(config, year, day)?;
    }

    Ok(())
}

/// Check the path provided for the specified line.
///
/// If the path specified does not exist, or does not contain that line, the line is appended.
/// A newline is added to the input line.
fn append_if_not_present<P, L>(path: P, line: L) -> Result<(), Error>
where
    P: AsRef<Path>,
    L: AsRef<[u8]>,
{
    // note that we have to work with the file as binary due to the possibility
    // that it is not utf-8.
    let mut line = line.as_ref().to_vec();
    line.push(b'\n');

    let contains_line = {
        std::fs::File::open(&path)
            .map(|file| {
                let mut reader = BufReader::new(file);
                let mut line_buffer = Vec::new();
                while let Ok(read_bytes) = reader.read_until(b'\n', &mut line_buffer) {
                    if read_bytes == 0 {
                        break;
                    }
                    if line_buffer == line {
                        return true;
                    }
                    line_buffer.clear();
                }
                false
            })
            .unwrap_or_default()
    };
    if !contains_line {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|err| Error::Io("opening for append", err))?;
        file.write_all(&line)
            .map_err(|err| Error::Io("appending", err))?;
    }
    Ok(())
}

/// Initialize a new year.
///
/// This entails:
///
/// - Configure various paths as desired.
/// - If implementation directory does not exist, or is empty, create a rust workspace there.
/// - Ensure the inputs directory is present in `"$implementation/.gitignore"` if it is a
///   subdirectory of the implementation.
pub fn initialize_year(config: &mut Config, year: u32, path_opts: PathOpts) -> Result<(), Error> {
    {
        // ensure all specified paths exist and are configured appropriately.
        let ensure_path = |maybe_path: Option<PathBuf>,
                           path_destination: &mut Option<PathBuf>|
         -> Result<(), Error> {
            match (&maybe_path, &path_destination) {
                (Some(desired_path), None) => {
                    // if we have a desired path and no appropriate path has already been configured,
                    // then:
                    if !desired_path.exists() {
                        std::fs::create_dir_all(&desired_path)
                            .map_err(|err| Error::Io("ensuring path dir", err))?;
                    }
                    *path_destination = Some(
                        desired_path
                            .canonicalize()
                            .map_err(|err| Error::Io("canonicalizing path destination", err))?,
                    );
                }
                (Some(desired_path), Some(configured_path))
                    if desired_path
                        .absolutize()
                        .map_err(|err| Error::Io("absolutizing desired path", err))?
                        != configured_path
                            .absolutize()
                            .map_err(|err| Error::Io("absolutizing configured path", err))? =>
                {
                    return Err(Error::ConfigCliConflict(
                        desired_path.display().to_string(),
                        configured_path.display().to_string(),
                    ));
                }
                _ => {
                    // take no action in any other case
                }
            }
            Ok(())
        };

        let paths = config.paths.entry(year).or_default();
        ensure_path(path_opts.input_files, &mut paths.input_files)?;
        ensure_path(path_opts.implementation, &mut paths.implementation)?;
        ensure_path(path_opts.day_templates, &mut paths.day_template)?;
    }

    let impl_path = config.implementation(year);

    // Create a new Rust project as required.
    // "Required" means that the target either does not exist, or is an empty directory.
    // This creates `Cargo.toml` and `.gitignore`.
    if !impl_path.exists()
        || (impl_path.is_dir()
            && std::fs::read_dir(&impl_path)
                .map(|mut dir_iter| dir_iter.next().is_none())
                .unwrap_or_default())
    {
        std::fs::create_dir_all(&impl_path)
            .map_err(|err| Error::Io("creating implementation dir", err))?;

        // Create default .gitignore
        append_if_not_present(impl_path.join(".gitignore"), "/target/")?;

        // create default `Cargo.toml` if not present.
        // Becuase `Cargo.toml` has more complicated semantics, we can't just append.
        if let Ok(file) = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(impl_path.join("Cargo.toml"))
        {
            let mut buffer = BufWriter::new(file);
            writeln!(&mut buffer, "[workspace]\nmembers = []")
                .map_err(|err| Error::Io("writing default Cargo.toml", err))?;
        }
    }

    // ensure inputs dir is in gitignore if it is (as per the default) a sub-directory of the
    // implementation dir
    if let Some(input_files_relative) =
        pathdiff::diff_paths(config.input_files(year), config.implementation(year))
    {
        // input files relative is a sub-directory of implementation dir
        // if not already present, add an appropriate ignore line
        if !input_files_relative.starts_with("..") {
            use std::os::unix::ffi::OsStrExt;

            // add a trailing slash to narrow the gitignore rule to directories
            let mut input_files_relative = input_files_relative.as_os_str().as_bytes().to_vec();
            input_files_relative.push(b'/');
            append_if_not_present(impl_path.join(".gitignore"), input_files_relative)?;
        }
    }

    Ok(())
}

/// Clear the templates directory.
///
/// This can be useful when the templates have been updated.
pub fn clear_templates(config: &Config, year: u32) -> Result<(), Error> {
    std::fs::remove_dir_all(config.day_template(year))
        .map_err(|err| Error::Io("attempting to clear templates", err))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(&'static str, #[source] std::io::Error),
    #[error("Cargo.toml not found")]
    NoCargoToml,
    #[error("could not parse Cargo.toml")]
    ParseToml(#[from] toml_edit::TomlError),
    #[error("Cargo.toml is malformed")]
    MalformedToml,
    #[error("failed to write updated Cargo.toml")]
    CargoTomlWrite(#[from] toml::ser::Error),
    #[error("template error for {1}")]
    Template(#[source] tinytemplate::error::Error, String),
    #[error("downloading input")]
    GetInput(#[from] aoclib::website::Error),
    #[error("crate already exists in workspace: {0}")]
    CrateAlreadyExists(String),
    #[error("building request client for day template download")]
    ClientBuilder(#[source] reqwest::Error),
    #[error("requesting day template file")]
    RequestingInput(#[source] reqwest::Error),
    #[error("response status unsuccessful requesting day template")]
    ResponseStatus(#[source] reqwest::Error),
    #[error("downloading day template to local file")]
    Downloading(#[source] reqwest::Error),
    #[error("CLI requested '{0}' but config file specified '{1}'")]
    ConfigCliConflict(String, String),
}

#[derive(StructOpt, Debug)]
pub struct PathOpts {
    /// Path to input files. Default: "$(pwd)/inputs"
    #[structopt(long, parse(from_os_str))]
    pub input_files: Option<PathBuf>,

    /// Path to this year's implementation directory. Default: "$(pwd)"
    #[structopt(long, parse(from_os_str))]
    pub implementation: Option<PathBuf>,

    /// Path to this year's day template files.
    #[structopt(long, parse(from_os_str))]
    pub day_templates: Option<PathBuf>,
}
