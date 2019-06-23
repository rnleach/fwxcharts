//
// API
//
pub use crate::{
    plot::{plot_all, save_all},
    sources::{
        load_all_sites_and_models, load_for_site_and_date_and_time, load_from_files, load_site,
        FileData,
    },
};

//
// Internal implementation details.
//
/// Types and functions for plotting
mod plot;
/// Functions for loading data from an archive or files.
mod sources;
/// Time series concepts such as `EnsembleList` and `TimeSeries` and transforms for applied
/// to those objects and for converting between them.
mod timeseries;
/// Types, like, `AnalyzedData`, `CapePartion` that are typically stored in
/// `TimeSeries`and the transformations between them.
mod types;
