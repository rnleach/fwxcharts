use bufcli::{ClimoDB, ClimoQueryInterface};
use bufkit_data::{Archive, Model};
use graphs::{load_site, plot_all};
use std::error::Error;

const DAYS_BACK: i64 = 2;

fn main() -> Result<(), Box<dyn Error>> {
    let home_dir = directories::UserDirs::new()
        .expect("No home directory!")
        .home_dir()
        .to_owned();
    let archive = home_dir.join("bufkit");
    let arch = Archive::connect(&archive)?;
    let climo = ClimoDB::connect_or_create(&archive)?;
    let climo = ClimoQueryInterface::initialize(&climo)?;

    let loaded_files = load_site(&arch, "KTUS", Model::GFS, DAYS_BACK).into_iter();

    plot_all(loaded_files, "images", Some(climo));

    Ok(())
}
