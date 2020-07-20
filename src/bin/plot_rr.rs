#![type_length_limit = "1115086"]
use bufcli::{ClimoDB, ClimoQueryInterface};
use bufkit_data::{Archive, Model, SiteInfo, StationNumber};
use chrono::{Duration, NaiveDate};
use graphs::{load_for_site_and_date_and_time, load_from_files, plot_all, FileData};
use std::error::Error;

const DAYS_BACK: i64 = 4;

fn main() -> Result<(), Box<dyn Error>> {
    let home_dir = directories::UserDirs::new()
        .expect("No UserDirs")
        .home_dir()
        .to_owned();
    let archive = home_dir.join("bufkit");
    let arch = Archive::connect(&archive)?;
    let climo = ClimoDB::connect_or_create(&archive)?;
    let climo = ClimoQueryInterface::initialize(&climo)?;

    let now = NaiveDate::from_ymd(2017, 9, 2).and_hms(12, 0, 0);

    let start_files = now;
    let end_files = now + Duration::days(3);

    let research_root = directories::UserDirs::new()
        .expect("No UserDirs")
        .document_dir()
        .expect("No document_dir")
        .join("Research")
        .join("2017 Fire")
        .join("Bufkit");

    let file_data = vec![
        FileData {
            site: SiteInfo {
                name: Some("KRR1".to_owned()),
                station_num: StationNumber::from(1),
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                research_root
                    .join("local_arw_krr1")
                    .join("2017090212.arw_krr1.buf"),
                research_root
                    .join("local_arw_krr1")
                    .join("2017090312.arw_krr1.buf"),
            ],
        },
        FileData {
            site: SiteInfo {
                name: Some("KRR2".to_owned()),
                station_num: StationNumber::from(2),
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                research_root
                    .join("local_arw_krr2")
                    .join("2017090212.arw_krr2.buf"),
                research_root
                    .join("local_arw_krr2")
                    .join("2017090312.arw_krr2.buf"),
            ],
        },
        FileData {
            site: SiteInfo {
                name: Some("KRR3".to_owned()),
                station_num: StationNumber::from(3),
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                research_root
                    .join("local_arw_krr3")
                    .join("2017090212.arw_krr3.buf"),
                research_root
                    .join("local_arw_krr3")
                    .join("2017090312.arw_krr3.buf"),
            ],
        },
        FileData {
            site: SiteInfo {
                name: Some("KRR4".to_owned()),
                station_num: StationNumber::from(4),
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                research_root
                    .join("local_arw_krr4")
                    .join("2017090212.arw_krr4.buf"),
                research_root
                    .join("local_arw_krr4")
                    .join("2017090312.arw_krr4.buf"),
            ],
        },
        FileData {
            site: SiteInfo {
                name: Some("KRR5".to_owned()),
                station_num: StationNumber::from(5),
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                research_root
                    .join("local_arw_krr5")
                    .join("2017090212.arw_krr5.buf"),
                research_root
                    .join("local_arw_krr5")
                    .join("2017090312.arw_krr5.buf"),
            ],
        },
        FileData {
            site: SiteInfo {
                name: Some("KSEE".to_owned()),
                station_num: StationNumber::from(6),
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                research_root
                    .join("local_arw_ksee")
                    .join("2017090212.arw_ksee.buf"),
                research_root
                    .join("local_arw_ksee")
                    .join("2017090312.arw_ksee.buf"),
            ],
        },
        FileData {
            site: SiteInfo {
                name: Some("KMSO".to_owned()),
                station_num: StationNumber::from(727730),
                notes: None,
                time_zone: None,
                state: None,
                auto_download: false,
            },
            model: "LocalWrf".to_owned(),
            start: start_files,
            end: end_files,
            files: vec![
                research_root
                    .join("local_arw_kmso")
                    .join("2017090212.arw_kmso.buf"),
                research_root
                    .join("local_arw_kmso")
                    .join("2017090312.arw_kmso.buf"),
            ],
        },
    ];

    let file_strings = file_data
        .into_iter()
        .map(load_from_files)
        .flat_map(|chan| chan.into_iter());

    let string_data = load_for_site_and_date_and_time(&arch, "kmso", Model::GFS, now, DAYS_BACK)
        .into_iter()
        .chain(
            load_for_site_and_date_and_time(&arch, "kmso", Model::NAM, now, DAYS_BACK).into_iter(),
        )
        .chain(
            load_for_site_and_date_and_time(&arch, "kmso", Model::NAM4KM, now, DAYS_BACK)
                .into_iter(),
        )
        .chain(
            load_for_site_and_date_and_time(&arch, "c18", Model::GFS, now, DAYS_BACK).into_iter(),
        )
        .chain(
            load_for_site_and_date_and_time(&arch, "c18", Model::NAM, now, DAYS_BACK).into_iter(),
        )
        .chain(
            load_for_site_and_date_and_time(&arch, "c18", Model::NAM4KM, now, DAYS_BACK)
                .into_iter(),
        )
        .chain(file_strings);

    plot_all(string_data, "images", Some(climo));

    Ok(())
}
