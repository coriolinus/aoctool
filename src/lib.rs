use serde::Serialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;
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
    let manifest = Document::from_str(&std::fs::read_to_string(&cargo_toml_path)?)?;

    Ok((cargo_toml_path, manifest))
}

fn add_crate_to_workspace(
    cargo_toml_path: &Path,
    manifest: &mut Document,
    crate_name: &str,
) -> Result<(), Error> {
    let root_table = manifest
        .root
        .as_table_mut()
        .expect("docuemnt root is a table");

    let workspace = root_table.entry("workspace");
    if workspace.is_none() {
        *workspace = toml_edit::Item::Table(toml_edit::Table::new());
    }
    let workspace = workspace.as_table_mut().ok_or(Error::MalformedToml)?;

    let members = workspace.entry("members");
    if members.is_none() {
        *members = toml_edit::Item::Value(toml_edit::Value::Array(Default::default()));
    }
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

    members.push(crate_name).map_err(|_| Error::MalformedToml)?;

    std::fs::write(cargo_toml_path, manifest.to_string_in_original_order())?;
    Ok(())
}

/// Ensure the template directory from the configuration exists and is initialized.
fn ensure_template_dir(config: &Config, year: u32) -> Result<PathBuf, Error> {
    let template_dir = config.day_template(year);
    if !template_dir.exists() {
        std::fs::create_dir_all(&template_dir)?;
    }
    for template in TEMPLATE_FILES {
        let template_path = template_dir.join(template);
        if !template_path.exists() {
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
                .open(template_path)?;
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
    use std::io::Write;

    #[derive(Serialize)]
    struct Context {
        day: u8,
        package_name: String,
    }

    let context = Context {
        day,
        package_name: day_name.to_string(),
    };

    // render templates
    let template_dir = ensure_template_dir(config, year)?;
    for template in TEMPLATE_FILES {
        let mut tt = TinyTemplate::new();
        let template_text = std::fs::read_to_string(template_dir.join(template))?;
        tt.add_template(template, &template_text)
            .map_err(|err| Error::Template(err, template.to_string()))?;
        let rendered_text = tt
            .render(template, &context)
            .map_err(|err| Error::Template(err, template.to_string()))?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(day_dir.join(template))?;
        file.write_all(rendered_text.as_bytes())?;
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
        std::fs::create_dir_all(day_dir.join("src"))?;

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

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
}
