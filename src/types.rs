//! This module contains types that would  normally be stored in a `TimeSeries` and operation that
//! would normally be performed on them or to create them.

use crate::timeseries::{ModelTimes, TimeSeries, ValidTime};
use chrono::{Duration, NaiveDateTime};

use sounding_analysis::Sounding;
use sounding_bufkit::BufkitData;

mod analyzed_data;
pub use analyzed_data::AnalyzedData;

impl ValidTime for Sounding {
    fn valid_time(&self) -> Option<NaiveDateTime> {
        self.valid_time()
    }
}

impl ModelTimes for Sounding {
    fn lead_time(&self) -> Option<Duration> {
        self.lead_time().map(|lt| Duration::hours(i64::from(lt)))
    }
}

impl<T: ValidTime> ValidTime for Vec<T> {
    // Assumes all items in the vector have the same valid time.
    fn valid_time(&self) -> Option<NaiveDateTime> {
        self.get(0).and_then(|t| t.valid_time())
    }
}

/// Parse a string into a `TimeSeries` of `sounding_analysis::Analysis` objects.
pub fn parse_sounding(
    str_data: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Option<TimeSeries<Sounding>> {
    BufkitData::init(&str_data, "")
        .ok()
        .map(|data| {
            data.into_iter()
                .filter(|(snd, _)| {
                    if let Some(vtime) = snd.valid_time() {
                        vtime >= start && vtime <= end
                    } else {
                        false
                    }
                })
                .map(|(snd, _)| snd)
                .collect::<Vec<Sounding>>()
        })
        .and_then(|vec_anals| {
            if vec_anals.is_empty() {
                None
            } else {
                Some(TimeSeries { data: vec_anals })
            }
        })
}
