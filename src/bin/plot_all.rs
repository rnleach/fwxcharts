use bufcli::{ClimoDB, ClimoQueryInterface};
use bufkit_data::Archive;
use graphs::{load_all_sites_and_models, plot_all};
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

    let string_data = load_all_sites_and_models(&arch, DAYS_BACK).into_iter();

    plot_all(string_data, "images", Some(climo));

    Ok(())
}
