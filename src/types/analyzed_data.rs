use crate::timeseries::{ModelTimes, ValidTime};
use chrono::{Duration, NaiveDateTime};

use metfor::{Celsius, CelsiusDiff, JpKg};
use sounding_analysis::{convective_parcel_initiation_energetics, hot_dry_windy, Analysis};

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

impl ValidTime for AnalyzedData {
    fn valid_time(&self) -> Option<NaiveDateTime> {
        Some(self.valid_time)
    }
}

impl ModelTimes for AnalyzedData {
    fn lead_time(&self) -> Option<Duration> {
        Some(Duration::hours(self.lead_time as i64))
    }
}

impl AnalyzedData {
    /// Convert a `sounding_analysis::Analysis` into an `AnalyzedData` struct.
    pub fn analyze(anal: &Analysis) -> Option<Self> {
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
}
