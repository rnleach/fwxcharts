use bufcli::{ClimoDB, ClimoQueryInterface};
use bufkit_data::{Archive, Model};
use graphs::{load_site, save_all};
use std::error::Error;

const DAYS_BACK: i64 = 2;
const ARCHIVE: &str = "/home/ryan/bufkit";

fn main() -> Result<(), Box<dyn Error>> {
    let arch = Archive::connect(ARCHIVE)?;
    let climo = ClimoDB::connect_or_create(ARCHIVE.as_ref())?;
    let climo = ClimoQueryInterface::initialize(&climo);

    let loaded_files = load_site(&arch, "KTUS", Model::GFS, DAYS_BACK)?.filter_map(Result::ok);

    save_all(loaded_files, "text", Some(climo))?;

    Ok(())
}
