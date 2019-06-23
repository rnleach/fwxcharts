use bufkit_data::Archive;
use graphs::{load_all_sites_and_models, plot_all};
use std::error::Error;

const DAYS_BACK: i64 = 2;
const ARCHIVE: &str = "/home/ryan/bufkit";

fn main() -> Result<(), Box<dyn Error>> {
    let arch = Archive::connect(ARCHIVE)?;

    let string_data = load_all_sites_and_models(&arch, DAYS_BACK)?.filter_map(Result::ok);

    plot_all(string_data, "images")?;

    Ok(())
}
