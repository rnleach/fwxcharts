//!
//! This module has functions for loading Bufkit data from an `Archive` or from file.
//!
//! These functions produce iterators suitable for the plot functions in this library.
//!
// TODO functions for loading files from disk.

use crate::timeseries::{EnsembleList, MetaData};
use bufkit_data::{Archive, BufkitDataErr, Model, Site};
use chrono::{Duration, NaiveDateTime, Utc};
use itertools::iproduct;
use std::{fs::File, io::Read, iter::once};
use strum::{AsStaticRef, IntoEnumIterator};

pub type StringData = EnsembleList<String>;

/// Information needed for making a plot from files on disk.
pub struct FileData {
    pub site: Site,
    pub model: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub files: Vec<String>,
}

/// Load the files from disk for plotting.
pub fn load_from_files<'a>(
    file_data: &'a FileData,
) -> Result<
    impl Iterator<Item = Result<StringData, bufkit_data::BufkitDataErr>> + 'a,
    bufkit_data::BufkitDataErr,
> {
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
                    .and_then(|anal| anal.sounding().valid_time())
                    .ok_or(BufkitDataErr::NotEnoughData)?;

                Ok((init_time, string))
            })
        })
        .collect();
    let strings = strings?;

    let string_data = StringData {
        meta,
        data: strings,
    };

    Ok(once(Ok(string_data)))
}

/// Load model initialization times for the given site and model assuming the current time is
/// the time given by the `time` parameter.
pub fn load_for_site_and_date_and_time<'a>(
    arch: &'a Archive,
    site: &str,
    model: Model,
    time: NaiveDateTime,
    days_back: i64,
) -> Result<
    impl Iterator<Item = Result<StringData, bufkit_data::BufkitDataErr>> + 'a,
    bufkit_data::BufkitDataErr,
> {
    let start = time - Duration::days(days_back);

    Ok(
        once((arch.site_info(site)?, model)).map(move |(site, model)| {
            let end = time + Duration::days(num_days(model));
            arch.init_times_for_soundings_valid_between(start, end, &site.id, model)
                .map(|init_times| {
                    let data = init_times
                        .into_iter()
                        .filter_map(|init_time| {
                            load_string_from_archive(arch, &site.id, init_time, model)
                                .map(|str_data| (init_time, str_data))
                        })
                        .collect::<Vec<_>>();
                    let meta = MetaData {
                        site,
                        model: model.as_static().to_owned(),
                        start,
                        now: time,
                        end,
                    };

                    StringData { meta, data }
                })
                .map_err(Into::into)
        }),
    )
}

/// Load all the model initialization times valid before now and going days back.
pub fn load_site<'a>(
    arch: &'a Archive,
    site: &str,
    model: Model,
    days_back: i64,
) -> Result<
    impl Iterator<Item = Result<StringData, bufkit_data::BufkitDataErr>> + 'a,
    bufkit_data::BufkitDataErr,
> {
    let now = Utc::now().naive_utc();

    load_for_site_and_date_and_time(arch, site, model, now, days_back)
}

/// Load all the model initialization itmes for all sites and models in the provided archive valid
/// before now and going days back.
pub fn load_all_sites_and_models<'a>(
    arch: &'a Archive,
    days_back: i64,
) -> Result<
    impl Iterator<Item = Result<StringData, bufkit_data::BufkitDataErr>> + 'a,
    bufkit_data::BufkitDataErr,
> {
    let now = Utc::now().naive_utc();
    let start = now - Duration::days(days_back);

    Ok(
        iproduct!(arch.sites()?.into_iter(), Model::iter()).map(move |(site, model)| {
            let end = now + Duration::days(num_days(model));
            arch.init_times_for_soundings_valid_between(start, end, &site.id, model)
                .map(|init_times| {
                    let data = init_times
                        .into_iter()
                        .filter_map(|init_time| {
                            load_string_from_archive(arch, &site.id, init_time, model)
                                .map(|str_data| (init_time, str_data))
                        })
                        .collect::<Vec<_>>();
                    let meta = MetaData {
                        site,
                        model: model.as_static().to_owned(),
                        start,
                        now,
                        end,
                    };

                    StringData { meta, data }
                })
                .map_err(Into::into)
        }),
    )
}

/// The number of days of data available for each model.
fn num_days(model: Model) -> i64 {
    match model {
        Model::GFS => 7,
        Model::NAM => 4,
        Model::NAM4KM => 3,
    }
}

fn load_string_from_archive(
    arch: &Archive,
    site: &str,
    init_time: NaiveDateTime,
    model: Model,
) -> Option<String> {
    arch.retrieve(site, model, init_time).ok()
}
