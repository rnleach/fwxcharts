use crate::timeseries::{ModelTimes, ValidTime};
use chrono::{Duration, NaiveDateTime};

use metfor::{CelsiusDiff, Meters};
use sounding_analysis::{experimental::fire::blow_up, hot_dry_windy, Sounding};

#[derive(Debug)]
pub struct AnalyzedData {
    pub valid_time: NaiveDateTime,
    pub lead_time: i32,
    pub hdw: f64,
    pub blow_up_dt: CelsiusDiff,
    pub blow_up_height: Meters,
}

impl ValidTime for AnalyzedData {
    fn valid_time(&self) -> Option<NaiveDateTime> {
        Some(self.valid_time)
    }
}

impl ModelTimes for AnalyzedData {
    fn lead_time(&self) -> Option<Duration> {
        Some(Duration::hours(i64::from(self.lead_time)))
    }
}

impl AnalyzedData {
    /// Convert a `sounding_analysis::Analysis` into an `AnalyzedData` struct.
    pub fn analyze(snd: &Sounding) -> Option<Self> {
        const MIN_BLOWUP: Meters = Meters(2000.0);
        const DEFAULT_BLOWUP: (CelsiusDiff, Meters) =
            (CelsiusDiff(std::f64::NAN), Meters(std::f64::NAN));

        let valid_time = snd.valid_time()?;
        let lead_time = snd.lead_time().into_option()?;

        let hdw = hot_dry_windy(snd).unwrap_or(std::f64::NAN);
        let (delta_t, height) = blow_up(snd, None)
            // Extract the values I need to plot
            .map(|bua| (bua.delta_t_lmib, bua.delta_z_lmib))
            // Plot a blank space (so use NAN marker) where there isn't a minimal blow up
            .map(|(dt, hgt)| {
                if hgt > MIN_BLOWUP {
                    (dt, hgt)
                } else {
                    DEFAULT_BLOWUP
                }
            })
            .unwrap_or(DEFAULT_BLOWUP);

        Some(AnalyzedData {
            valid_time,
            lead_time,
            hdw,
            blow_up_dt: delta_t,
            blow_up_height: height,
        })
    }
}
