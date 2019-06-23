use bufkit_data::{Archive, Model};
use graphs::{load_site, plot_all};
use std::error::Error;

const DAYS_BACK: i64 = 2;
const ARCHIVE: &str = "/home/ryan/bufkit";

fn main() -> Result<(), Box<dyn Error>> {
    let arch = Archive::connect(ARCHIVE)?;

    let loaded_files = load_site(&arch, "KTUS", Model::GFS, DAYS_BACK)?.filter_map(Result::ok);

    plot_all(loaded_files, "images")?;

    Ok(())
}
