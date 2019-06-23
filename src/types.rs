//! This module contains types that would  normally be stored in a `TimeSeries` and operation that
//! would normally be performed on them or to create them.

use crate::timeseries::{ModelTimes, TimeSeries, ValidTime};
use chrono::{Duration, NaiveDateTime};

use metfor::{Celsius, CelsiusDiff, JpKg};
use sounding_analysis::{
    convective_parcel_initiation_energetics, hot_dry_windy, lift_parcel, mixed_layer_parcel,
    partition_cape, Analysis, Parcel,
};
use sounding_bufkit::BufkitData;

/// Data format for dT, dry cape, and wet cape used in plots.
#[derive(Debug)]
pub struct CapePartition {
    pub valid_time: NaiveDateTime,
    pub dt: CelsiusDiff,
    pub dry: JpKg,
    pub wet: JpKg,
}

#[derive(Debug)]
pub struct AnalyzedData {
    pub valid_time: NaiveDateTime,
    pub lead_time: i32,
    pub hdw: f64,
    pub t0: Celsius,
    pub dt0: CelsiusDiff,
    pub e0: JpKg,
    pub de: JpKg,
}

impl ValidTime for Analysis {
    fn valid_time(&self) -> Option<NaiveDateTime> {
        self.sounding().valid_time()
    }
}
impl ModelTimes for Analysis {
    fn lead_time(&self) -> Option<Duration> {
        self.sounding()
            .lead_time()
            .map(|lt| Duration::hours(i64::from(lt)))
    }
}

impl ValidTime for CapePartition {
    fn valid_time(&self) -> Option<NaiveDateTime> {
        Some(self.valid_time)
    }
}

impl ValidTime for AnalyzedData {
    fn valid_time(&self) -> Option<NaiveDateTime> {
        Some(self.valid_time)
    }
}

impl<T: ValidTime> ValidTime for Vec<T> {
    // Assumes all items in the vector have the same valid time.
    fn valid_time(&self) -> Option<NaiveDateTime> {
        self.get(0).and_then(|t| t.valid_time())
    }
}

impl ModelTimes for AnalyzedData {
    fn lead_time(&self) -> Option<Duration> {
        Some(Duration::hours(self.lead_time as i64))
    }
}

/// Parse a string into a `TimeSeries` of `sounding_analysis::Analysis` objects.
pub fn parse_sounding(
    str_data: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Option<TimeSeries<Analysis>> {
    BufkitData::init(&str_data, "")
        .ok()
        .map(|data| {
            data.into_iter()
                .filter(|anal| {
                    if let Some(vtime) = anal.sounding().valid_time() {
                        vtime >= start && vtime <= end
                    } else {
                        false
                    }
                })
                .collect::<Vec<Analysis>>()
        })
        .and_then(|vec_anals| {
            if vec_anals.is_empty() {
                None
            } else {
                Some(TimeSeries { data: vec_anals })
            }
        })
}

/// Convert an `sounding_analysis::Analysis` into a vector of `CapePartition`s.
pub fn analyze_cape_partitions(anal: &Analysis) -> Option<Vec<CapePartition>> {
    const NUM_DT: usize = 60;
    const MAX_DT: f64 = 15.0;

    let snd = anal.sounding();
    let valid_time = snd.valid_time()?;
    let mut cape_partitions = Vec::with_capacity(NUM_DT);
    let starting_parcel = mixed_layer_parcel(snd).ok()?;

    for i in 0..=NUM_DT {
        let dt = CelsiusDiff(MAX_DT / NUM_DT as f64 * i as f64);
        let parcel = Parcel {
            temperature: starting_parcel.temperature + dt,
            ..starting_parcel
        };
        let (dry, wet) = lift_parcel(parcel, snd)
            .and_then(|pa| partition_cape(&pa))
            .unwrap_or((JpKg(std::f64::NAN), JpKg(std::f64::NAN)));

        cape_partitions.push(CapePartition {
            valid_time,
            dt,
            dry,
            wet,
        });
    }

    Some(cape_partitions)
}

/// Convert a `sounding_analysis::Analysis` into an `AnalyzedData` struct.
pub fn analyze(anal: &Analysis) -> Option<AnalyzedData> {
    let snd = anal.sounding();
    let valid_time = snd.valid_time()?;
    let lead_time = snd.lead_time().into_option()?;

    let hdw = hot_dry_windy(snd).unwrap_or(std::f64::NAN);
    let (t0, dt0, e0, de) = convective_parcel_initiation_energetics(snd).unwrap_or((
        Celsius(std::f64::NAN),
        CelsiusDiff(std::f64::NAN),
        JpKg(std::f64::NAN),
        JpKg(std::f64::NAN),
    ));

    Some(AnalyzedData {
        valid_time,
        lead_time,
        hdw,
        t0,
        dt0,
        e0,
        de,
    })
}
