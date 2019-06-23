use bufkit_data::Site;
use chrono::{Duration, NaiveDateTime};
use std::collections::hash_map::{Entry, HashMap};

/// `MetaData` contains information about when the associated data should start and stop, what time
/// is considered now, the site, and the model name for which the associated data is valid for.
#[derive(Clone, Debug, PartialEq)]
pub struct MetaData {
    pub site: Site,
    pub model: String,
    pub start: NaiveDateTime,
    pub now: NaiveDateTime,
    pub end: NaiveDateTime,
}

/// `ValidTime` is a trait that means an object has a "valid time", or a specific time that it
/// is valid.
pub trait ValidTime {
    /// Get the valid time
    fn valid_time(&self) -> Option<NaiveDateTime>;
}

/// `ModelTimes` is a trait that means an object has a "valid time" and a "lead time".
///
/// Lead time is in terms of a forecast.
pub trait ModelTimes: ValidTime {
    /// Get the lead time
    fn lead_time(&self) -> Option<Duration>;
}

/// `TimeSeries` is a wrapper around a `std::vec::Vec` with elements that are sorted by their
///  valid times.
pub struct TimeSeries<T: ValidTime> {
    pub data: Vec<T>,
}

/// `EnsembleList` contains a `MetaData` and a list of data items each associated with an
/// initialization time, i.e. a model initialization time.
pub struct EnsembleList<T> {
    pub meta: MetaData,
    pub data: Vec<(NaiveDateTime, T)>,
}

/// `EnsembleSeries` is the special case of an `EnsembleList` where the contained data type is a
/// `TimeSeries`.
pub type EnsembleSeries<T> = EnsembleList<TimeSeries<T>>;

/// `MergedSeries` contains a `MetaData` and a `TimeSeries`. It may represent a single model run
/// or an ensemble of model runs with different initialization times merged into a single time
/// series where for any valid time the ensemble member with the shortest lead time selected for
/// the time series.
pub struct MergedSeries<T: ValidTime> {
    pub meta: MetaData,
    pub data: TimeSeries<T>,
}

impl<T> EnsembleList<T> {
    /// Map and filter out errors.
    pub fn filter_map<U, F>(&self, func: F) -> EnsembleList<U>
    where
        F: Fn(&T) -> Option<U>,
    {
        let EnsembleList { meta, data } = &self;

        let data: Vec<(NaiveDateTime, U)> = data
            .iter()
            .filter_map(|(init_time, t)| func(t).map(|u| (*init_time, u)))
            .collect();

        EnsembleList {
            meta: meta.clone(),
            data,
        }
    }

    /// Check if the data member is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T: ValidTime> EnsembleSeries<T> {
    /// Map and filter out errors.
    pub fn filter_map_inner<U, F>(&self, func: F) -> EnsembleSeries<U>
    where
        F: Fn(&T) -> Option<U>,
        U: ValidTime,
    {
        let EnsembleSeries { meta, data } = &self;

        let data: Vec<(NaiveDateTime, TimeSeries<U>)> = data
            .iter()
            .filter_map(|(init_time, vec_t)| {
                let inner_data: Vec<U> = vec_t
                    .as_ref()
                    .iter()
                    .filter_map(|val_t| func(val_t))
                    .collect();

                let inner_data = TimeSeries { data: inner_data };

                if inner_data.as_ref().is_empty() {
                    None
                } else {
                    Some((*init_time, inner_data))
                }
            })
            .collect();

        EnsembleSeries {
            meta: meta.clone(),
            data,
        }
    }
}

impl<T: ValidTime> MergedSeries<T> {
    /// Map and filter out errors.
    pub fn filter_map<U, F>(&self, func: F) -> MergedSeries<U>
    where
        F: Fn(&T) -> Option<U>,
        U: ValidTime,
    {
        let MergedSeries { meta, data } = &self;

        let data: Vec<U> = data
            .as_ref()
            .iter()
            .filter_map(|val_t| func(val_t))
            .collect();
        let data = TimeSeries { data };

        MergedSeries {
            meta: meta.clone(),
            data,
        }
    }

    /// Check if the data member is empty.
    pub fn is_empty(&self) -> bool {
        self.data.as_ref().is_empty()
    }
}

impl<T: ValidTime> AsRef<[T]> for TimeSeries<T> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

impl<T: ModelTimes> EnsembleSeries<T> {
    /// Transform an `EnsembleSeries` into a `MergedSeries`.
    ///
    /// Assumes the EnsembleSeries is sorted in order of ascending model initialization time.
    pub fn merge(self) -> MergedSeries<T> {
        let EnsembleSeries { meta, data } = self;

        let mut pool: HashMap<NaiveDateTime, T> = HashMap::new();

        data.into_iter().for_each(|(_init_time, time_series_t)| {
            let TimeSeries { data: vec_t } = time_series_t;

            vec_t.into_iter().for_each(|val_t| {
                if let (Some(valid_time), Some(lead_time)) = (val_t.valid_time(), val_t.lead_time())
                {
                    match pool.entry(valid_time) {
                        Entry::Occupied(mut entry) => {
                            let cmp_val = entry.get_mut();
                            if lead_time < cmp_val.lead_time().unwrap() {
                                *cmp_val = val_t;
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(val_t);
                        }
                    }
                }
            });
        });

        let mut data: Vec<T> = pool.into_iter().map(|(_k, v)| v).collect();
        data.sort_by_key(|val| val.valid_time());
        let data = TimeSeries { data };

        MergedSeries { meta, data }
    }
}
