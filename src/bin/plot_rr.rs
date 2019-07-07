#![type_length_limit = "1115086"]
use bufcli::{ClimoDB, ClimoQueryInterface};
use bufkit_data::{Archive, Model, Site};
use chrono::{Duration, NaiveDate};
use graphs::{load_for_site_and_date_and_time, load_from_files, plot_all, FileData};
use std::error::Error;

const DAYS_BACK: i64 = 4;
const ARCHIVE: &str = "/home/ryan/bufkit";

fn main() -> Result<(), Box<dyn Error>> {
    let arch = Archive::connect(ARCHIVE)?;
    let climo = ClimoDB::connect_or_create(ARCHIVE.as_ref())?;
    let climo = ClimoQueryInterface::initialize(&climo);

    let now = NaiveDate::from_ymd(2017, 9, 2).and_hms(12, 0, 0);

    let start_files = now;
    let end_files = now + Duration::days(3);

    let file_data = [
        FileData {
            site: Site {
                id: "KRR1".to_owned(),
                name: None,
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr1/2017090212.arw_krr1.buf"
                    .to_owned(),
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr1/2017090312.arw_krr1.buf"
                    .to_owned(),
            ],
        },
        FileData {
            site: Site {
                id: "KRR2".to_owned(),
                name: None,
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr2/2017090212.arw_krr2.buf"
                    .to_owned(),
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr2/2017090312.arw_krr2.buf"
                    .to_owned(),
            ],
        },
        FileData {
            site: Site {
                id: "KRR3".to_owned(),
                name: None,
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr3/2017090212.arw_krr3.buf"
                    .to_owned(),
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr3/2017090312.arw_krr3.buf"
                    .to_owned(),
            ],
        },
        FileData {
            site: Site {
                id: "KRR4".to_owned(),
                name: None,
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr4/2017090212.arw_krr4.buf"
                    .to_owned(),
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr4/2017090312.arw_krr4.buf"
                    .to_owned(),
            ],
        },
        FileData {
            site: Site {
                id: "KRR5".to_owned(),
                name: None,
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr5/2017090212.arw_krr5.buf"
                    .to_owned(),
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_krr5/2017090312.arw_krr5.buf"
                    .to_owned(),
            ],
        },
        FileData {
            site: Site {
                id: "KSEE".to_owned(),
                name: None,
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_ksee/2017090212.arw_ksee.buf"
                    .to_owned(),
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_ksee/2017090312.arw_ksee.buf"
                    .to_owned(),
            ],
        },
        FileData {
            site: Site {
                id: "KMSO".to_owned(),
                name: None,
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_kmso/2017090212.arw_kmso.buf"
                    .to_owned(),
                "/home/ryan/Documents/2017 Fire/Bufkit/local_arw_kmso/2017090312.arw_kmso.buf"
                    .to_owned(),
            ],
        },
    ];

    let file_strings = file_data
        .iter()
        .filter_map(|fd| load_from_files(fd).ok())
        .flat_map(|iter| iter);

    let string_data = load_for_site_and_date_and_time(&arch, "kmso", Model::GFS, now, DAYS_BACK)?
        .chain(load_for_site_and_date_and_time(
            &arch,
            "kmso",
            Model::NAM,
            now,
            DAYS_BACK,
        )?)
        .chain(load_for_site_and_date_and_time(
            &arch,
            "kmso",
            Model::NAM4KM,
            now,
            DAYS_BACK,
        )?)
        .chain(load_for_site_and_date_and_time(
            &arch,
            "c18",
            Model::GFS,
            now,
            DAYS_BACK,
        )?)
        .chain(load_for_site_and_date_and_time(
            &arch,
            "c18",
            Model::NAM,
            now,
            DAYS_BACK,
        )?)
        .chain(load_for_site_and_date_and_time(
            &arch,
            "c18",
            Model::NAM4KM,
            now,
            DAYS_BACK,
        )?)
        .chain(file_strings)
        .filter_map(Result::ok);

    plot_all(string_data, "images", Some(climo))?;

    Ok(())
}
