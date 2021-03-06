//!
//! This module has functions for loading Bufkit data from an `Archive` or from file.
//!
//! These functions produce iterators suitable for the plot functions in this library.
//!

use crate::{
    messages::{InnerMessage, Message},
    timeseries::{EnsembleList, MetaData},
};
use bufkit_data::{Archive, BufkitDataErr, Model, SiteInfo};
use chrono::{Duration, NaiveDateTime, Utc};
use crossbeam::crossbeam_channel::{unbounded, Receiver};
use std::{fs::File, io::Read, thread::spawn};
use strum::IntoEnumIterator;

pub type StringData = EnsembleList<String>;

/// Information needed for making a plot from files on disk.
pub struct FileData {
    pub site: SiteInfo,
    pub model: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub files: Vec<std::path::PathBuf>,
}

/// Load the files from disk for plotting.
pub fn load_from_files(file_data: FileData) -> Receiver<Message> {
    let (sender, receiver) = unbounded();

    spawn(move || {
        let meta = MetaData {
            site: file_data.site.clone(),
            model: file_data.model.clone(),
            start: file_data.start,
            now: file_data.start,
            end: file_data.end,
        };

        let strings: Result<Vec<(NaiveDateTime, String)>, _> = file_data
            .files
            .iter()
            .map(|path| {
                let mut f = File::open(path)?;
                let mut contents = String::new();
                f.read_to_string(&mut contents)?;
                Ok(contents)
            })
            .map(|res: Result<String, std::io::Error>| res.map_err(BufkitDataErr::from))
            .map(|res| {
                res.and_then(|string| {
                    let init_time: NaiveDateTime = sounding_bufkit::BufkitData::init(&string, "")
                        .map_err(BufkitDataErr::from)?
                        .into_iter()
                        .nth(0)
                        .and_then(|(snd, _)| snd.valid_time())
                        .ok_or(BufkitDataErr::NotEnoughData)?;

                    Ok((init_time, string))
                })
            })
            .collect();

        match strings {
            Ok(strings) => {
                let msg = InnerMessage::StringData(StringData {
                    meta,
                    data: strings,
                });

                sender.send(Message::from(msg)).unwrap();
            }
            Err(err) => {
                let msg = InnerMessage::BufkitDataError(err);
                sender.send(Message::from(msg)).unwrap();
            }
        }
    });

    receiver
}

/// Load model initialization times for the given site and model assuming the current time is
/// the time given by the `time` parameter.
pub fn load_for_site_and_date_and_time<'a>(
    arch: &'a Archive,
    site: &str,
    model: Model,
    time: NaiveDateTime,
    days_back: i64,
) -> Receiver<Message> {
    let root = arch.root().to_path_buf();
    let site = site.to_owned();
    let (sender, receiver) = unbounded();

    spawn(move || {
        let arch = match Archive::connect(&root) {
            Ok(arch) => arch,
            Err(err) => {
                sender
                    .send(Message::from(InnerMessage::BufkitDataError(err)))
                    .unwrap();
                return;
            }
        };

        let start = time - Duration::days(days_back);
        let end = time + Duration::days(num_days(model));
        let site_info = match arch
            .station_num_for_id(&site, model)
            .and_then(|stn_num| arch.site(stn_num).ok_or(BufkitDataErr::NotInIndex))
        {
            Ok(site_info) => site_info,
            Err(err) => {
                sender
                    .send(Message::from(InnerMessage::BufkitDataError(err)))
                    .unwrap();
                return;
            }
        };

        match arch.retrieve_all_valid_in(site_info.station_num, model, start, end) {
            Ok(data) => {
                let data: Vec<(NaiveDateTime, String)> = data
                    .filter_map(|string| {
                        let init_time: NaiveDateTime =
                            sounding_bufkit::BufkitData::init(&string, "")
                                .ok()?
                                .into_iter()
                                .nth(0)
                                .and_then(|(snd, _)| snd.valid_time())?;

                        Some((init_time, string))
                    })
                    .collect();

                let meta = MetaData {
                    site: site_info,
                    model: model.as_static_str().to_owned(),
                    start,
                    now: time,
                    end,
                };

                let msg = InnerMessage::StringData(StringData { meta, data });

                sender.send(Message::from(msg)).unwrap();
            }
            Err(err) => {
                sender
                    .send(Message::from(InnerMessage::BufkitDataError(err)))
                    .unwrap();
            }
        }
    });

    receiver
}

/// Load all the model initialization times valid before now and going days back.
pub fn load_site<'a>(
    arch: &'a Archive,
    site: &str,
    model: Model,
    days_back: i64,
) -> Receiver<Message> {
    let now = Utc::now().naive_utc();

    load_for_site_and_date_and_time(arch, site, model, now, days_back)
}

/// Load all the model initialization times for all sites and models in the provided archive valid
/// before now and going days back.
pub fn load_all_sites_and_models(arch: &Archive, days_back: i64) -> Receiver<Message> {
    let root = arch.root().to_path_buf();
    let (sender, receiver) = unbounded();

    spawn(move || {
        let arch = match Archive::connect(&root) {
            Ok(arch) => arch,
            Err(err) => {
                sender
                    .send(Message::from(InnerMessage::BufkitDataError(err)))
                    .unwrap();
                return;
            }
        };

        let now = Utc::now().naive_utc();
        let start = now - Duration::days(days_back);

        for model in Model::iter() {
            let sites_ids = match arch.sites_and_ids_for(model) {
                Ok(sites_ids) => sites_ids,
                Err(err) => {
                    sender
                        .send(Message::from(InnerMessage::BufkitDataError(err)))
                        .unwrap();
                    return;
                }
            };

            let end = now + Duration::days(num_days(model));

            for (site_info, _site_id) in sites_ids.into_iter() {
                match arch.retrieve_all_valid_in(site_info.station_num, model, start, end) {
                    Ok(data) => {
                        let data: Vec<(NaiveDateTime, String)> = data
                            .filter_map(|string| {
                                let init_time: NaiveDateTime =
                                    sounding_bufkit::BufkitData::init(&string, "")
                                        .ok()?
                                        .into_iter()
                                        .nth(0)
                                        .and_then(|(snd, _)| snd.valid_time())?;

                                Some((init_time, string))
                            })
                            .collect();

                        let meta = MetaData {
                            site: site_info,
                            model: model.as_static_str().to_owned(),
                            start,
                            now,
                            end,
                        };

                        let msg = InnerMessage::StringData(StringData { meta, data });

                        sender.send(Message::from(msg)).unwrap();
                    }
                    Err(err) => {
                        sender
                            .send(Message::from(InnerMessage::BufkitDataError(err)))
                            .unwrap();
                    }
                }
            }
        }
    });

    receiver
}

/// The number of days of data available for each model.
fn num_days(model: Model) -> i64 {
    match model {
        Model::GFS => 7,
        Model::NAM => 4,
        Model::NAM4KM => 3,
    }
}
