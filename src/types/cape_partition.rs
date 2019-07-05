use crate::timeseries::ValidTime;
use chrono::NaiveDateTime;

use metfor::{CelsiusDiff, JpKg};
use sounding_analysis::{lift_parcel, mixed_layer_parcel, partition_cape, Analysis, Parcel};

/// Data format for dT, dry cape, and wet cape used in plots.
#[derive(Debug)]
pub struct CapePartition {
    pub valid_time: NaiveDateTime,
    pub dt: CelsiusDiff,
    pub dry: JpKg,
    pub wet: JpKg,
}

impl ValidTime for CapePartition {
    fn valid_time(&self) -> Option<NaiveDateTime> {
        Some(self.valid_time)
    }
}

impl CapePartition {
    /// Convert an `sounding_analysis::Analysis` into a vector of `CapePartition`s.
    pub fn analyze_cape_partitions(anal: &Analysis) -> Option<Vec<Self>> {
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
}
