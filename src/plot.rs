//! Functions used for plotting data and producing output.

use crate::{
    sources::StringData,
    timeseries::{EnsembleSeries, MergedSeries, MetaData},
    types::{analyze, analyze_cape_partitions, parse_sounding, AnalyzedData, CapePartition},
};
use metfor::Quantity;
use std::{
    error::Error,
    fs::File,
    io::Write,
    path::PathBuf,
    process::{ChildStdin, Command, Stdio},
};

/// Given an iterator over `StringData` loaded from Bufkit files, filter out any failed results
/// and make all the plots.
///
/// # Arguments
/// iter - an iterator over ensembles of model runs, make the plot and save it for each ensemble.
/// prefix - The path to the folder where you want the plots saved.
pub fn plot_all(
    iter: impl Iterator<Item = StringData>,
    prefix: &str,
) -> Result<(), Box<dyn Error>> {
    let gp_in = &mut launch_gnuplot(prefix)?;

    iter.filter_map(|ens_list_strings| {
        let start = ens_list_strings.meta.start;
        let end = ens_list_strings.meta.end;
        let ens_ser_anal =
            ens_list_strings.filter_map(|str_data| parse_sounding(str_data, start, end));

        if ens_ser_anal.is_empty() {
            None
        } else {
            Some(ens_ser_anal)
        }
    })
    .map(|ens_ser_anal| {
        let analyzed_data = ens_ser_anal.filter_map_inner(|anal| analyze(anal));
        (ens_ser_anal, analyzed_data)
    })
    .map(|(ens_ser_anal, analyzed_data)| {
        let merged_ser_anal = ens_ser_anal.merge();
        (merged_ser_anal, analyzed_data)
    })
    .map(|(merged_ser_anal, analyzed_data)| {
        let cape_parts = merged_ser_anal.filter_map(|anal| analyze_cape_partitions(anal));
        (analyzed_data, cape_parts)
    })
    .for_each(|(analyzed_data, cape_parts)| {
        gp_plot_ens(gp_in, &analyzed_data).unwrap_or(());
        let merged = analyzed_data.merge();
        gp_plot_mrg(gp_in, &merged, &cape_parts).unwrap_or(());
    });

    Ok(())
}

/// Given an iterator over `StringData` loaded from Bufkit files, filter out any failed results
/// and save the data in files suitable for gnuplot.
///
/// # Arguments
/// iter - an iterator over ensembles of model runs, make the plot and save it for each ensemble.
/// prefix - The path to the folder where you want the plots saved.
pub fn save_all(
    iter: impl Iterator<Item = StringData>,
    prefix: &str,
) -> Result<(), Box<dyn Error>> {
    iter.filter_map(|ens_list_strings| {
        let start = ens_list_strings.meta.start;
        let end = ens_list_strings.meta.end;
        let ens_ser_anal =
            ens_list_strings.filter_map(|str_data| parse_sounding(str_data, start, end));

        if ens_ser_anal.is_empty() {
            None
        } else {
            Some(ens_ser_anal)
        }
    })
    .map(|ens_ser_anal| {
        let analyzed_data = ens_ser_anal.filter_map_inner(|anal| analyze(anal));
        (ens_ser_anal, analyzed_data)
    })
    .map(|(ens_ser_anal, analyzed_data)| {
        let merged_ser_anal = ens_ser_anal.merge();
        (merged_ser_anal, analyzed_data)
    })
    .map(|(merged_ser_anal, analyzed_data)| {
        let cape_parts = merged_ser_anal.filter_map(|anal| analyze_cape_partitions(anal));
        (analyzed_data, cape_parts)
    })
    .for_each(|(analyzed_data, cape_parts)| {
        gp_save(prefix, analyzed_data, cape_parts).unwrap_or(())
    });

    Ok(())
}

const GP_INIT: &str = include_str!("plot/initialize.plt");
const GP_PLOT_ENS: &str = include_str!("plot/ens_template.plt");
const GP_PLOT_MRG: &str = include_str!("plot/mrg_template.plt");
const GP_PLOT: &str = include_str!("plot/summary_template.plt");
const GP_DATE_FORMAT: &str = "%Y-%m-%d-%H";

/// Create a pipe to a gnuplot process and set up the terminal, etc
///
/// output_prefix is a path to a folder to put the images in when completed.
fn launch_gnuplot(output_prefix: &str) -> Result<ChildStdin, Box<dyn Error>> {
    let gp = Command::new("gnuplot")
        .arg("-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut gp_in = gp.stdin.expect("no stdin assigned, should be impossible!");
    gp_in.write_all(GP_INIT.as_bytes())?;
    writeln!(gp_in, "output_prefix=\"{}\"", output_prefix)?;

    Ok(gp_in)
}

/// Plot a merged time series, including a heat map.
fn gp_plot_mrg(
    gp: &mut ChildStdin,
    mg: &MergedSeries<AnalyzedData>,
    cape_parts: &MergedSeries<Vec<CapePartition>>,
) -> Result<(), Box<dyn Error>> {
    let MergedSeries::<AnalyzedData> { meta: meta_mg, .. } = &mg;
    let MergedSeries::<Vec<CapePartition>> {
        meta: meta_cape, ..
    } = &cape_parts;

    assert_eq!(meta_cape, meta_mg);

    // Set variables for the gnuplot script to use for ranges, etc
    writeln!(gp, "num_hours={}", (meta_mg.end - meta_mg.now).num_hours())?;
    writeln!(gp, "now_time=\"{}\"", meta_mg.now.format(GP_DATE_FORMAT),)?;
    writeln!(
        gp,
        "start_time=\"{}\"",
        meta_mg.start.format(GP_DATE_FORMAT)
    )?;
    writeln!(gp, "end_time=\"{}\"", meta_mg.end.format(GP_DATE_FORMAT))?;
    writeln!(
        gp,
        "main_title=\"Fire Weather Parameters - {} - {}\"",
        meta_mg.site.name.as_ref().unwrap_or(&meta_mg.site.id),
        meta_mg.model.to_uppercase()
    )?;
    writeln!(
        gp,
        "output_name=\"{}_{}\"",
        meta_mg.site.id,
        meta_mg.model.to_uppercase()
    )?;

    writeln!(gp, "$data << EOD")?;
    write_merged_data(mg, gp)?;
    writeln!(gp, "EOD")?;

    // Write out the merged time series data for the heat map
    writeln!(gp, "$wet_dry_data << EOD")?;
    write_merged_heat_map_data(cape_parts, gp)?;
    writeln!(gp, "EOD")?;

    // Draw the graph
    gp.write_all(GP_PLOT_MRG.as_bytes())?;

    Ok(())
}

/// Plot a set of ensemble data
fn gp_plot_ens(
    gp: &mut ChildStdin,
    ens: &EnsembleSeries<AnalyzedData>,
) -> Result<(), Box<dyn Error>> {
    let EnsembleSeries::<AnalyzedData> { meta, .. } = ens;

    // Set variables for the gnuplot script to use for ranges, etc
    writeln!(gp, "num_hours={}", (meta.end - meta.now).num_hours())?;
    writeln!(gp, "now_time=\"{}\"", meta.now.format(GP_DATE_FORMAT),)?;
    writeln!(gp, "start_time=\"{}\"", meta.start.format(GP_DATE_FORMAT))?;
    writeln!(gp, "end_time=\"{}\"", meta.end.format(GP_DATE_FORMAT))?;
    writeln!(
        gp,
        "main_title=\"Fire Weather Parameters - {} - {}\"",
        meta.site.name.as_ref().unwrap_or(&meta.site.id),
        meta.model.to_uppercase()
    )?;
    writeln!(
        gp,
        "output_name=\"{}_{}_ens.png\"",
        meta.site.id,
        meta.model.to_uppercase()
    )?;

    // Write out the ensemble data
    writeln!(gp, "$data << EOD")?;
    write_ensemble_data(&ens, gp)?;
    writeln!(gp, "EOD")?;

    // Draw the graph
    gp.write_all(GP_PLOT_ENS.as_bytes())?;

    Ok(())
}

/// Plot a set of data
#[deprecated]
fn gp_plot_old(
    gp: &mut ChildStdin,
    ens: EnsembleSeries<AnalyzedData>,
    mg: MergedSeries<Vec<CapePartition>>,
) -> Result<(), Box<dyn Error>> {
    let EnsembleSeries::<AnalyzedData> { meta, .. } = &ens;
    let MergedSeries::<Vec<CapePartition>> { meta: meta_mg, .. } = &mg;

    assert_eq!(meta, meta_mg);

    //
    // Set variables for the gnuplot script to use for ranges, etc
    //
    writeln!(gp, "num_hours={}", (meta.end - meta.now).num_hours())?;
    writeln!(gp, "now_time=\"{}\"", meta.now.format(GP_DATE_FORMAT),)?;
    writeln!(gp, "start_time=\"{}\"", meta.start.format(GP_DATE_FORMAT))?;
    writeln!(gp, "end_time=\"{}\"", meta.end.format(GP_DATE_FORMAT))?;
    writeln!(
        gp,
        "main_title=\"Fire Weather Parameters - {} - {}\"",
        meta.site.name.as_ref().unwrap_or(&meta.site.id),
        meta.model.to_uppercase()
    )?;
    writeln!(
        gp,
        "output_name=\"{}_{}.png\"",
        meta.site.id,
        meta.model.to_uppercase()
    )?;

    // Write out the ensemble data
    writeln!(gp, "$data << EOD")?;
    write_ensemble_data(&ens, gp)?;
    writeln!(gp, "EOD")?;

    // Make a merged data and write that out too.
    let merged = ens.merge();

    writeln!(gp, "$merged_data << EOD")?;
    write_merged_data(&merged, gp)?;
    writeln!(gp, "EOD")?;

    // Write out the merged time series data for the heat map
    writeln!(gp, "$wet_dry_data << EOD")?;
    write_merged_heat_map_data(&mg, gp)?;
    writeln!(gp, "EOD")?;

    // Draw the graph
    gp.write_all(GP_PLOT.as_bytes())?;

    println!("Finished plot {}/{}", meta_mg.site.id, meta_mg.model);

    Ok(())
}

/// Save a set of data
fn gp_save(
    prefix: &str,
    ens: EnsembleSeries<AnalyzedData>,
    mg: MergedSeries<Vec<CapePartition>>,
) -> Result<(), Box<dyn Error>> {
    let EnsembleSeries::<AnalyzedData> { meta, .. } = &ens;
    let MergedSeries::<Vec<CapePartition>> { meta: meta_mg, .. } = &mg;

    assert_eq!(meta, meta_mg);

    // Build the file names to save the data to
    let fname_ens: PathBuf = PathBuf::from(&format!(
        "{}/{}_{}_ens.dat",
        prefix,
        meta.site.id,
        meta.model.to_uppercase()
    ));
    let f_ens = &mut File::create(&fname_ens)?;
    let fname_mrg: PathBuf = PathBuf::from(&format!(
        "{}/{}_{}_mrg.dat",
        prefix,
        meta.site.id,
        meta.model.to_uppercase()
    ));
    let f_mrg = &mut File::create(&fname_mrg)?;
    let fname_hm: PathBuf = PathBuf::from(&format!(
        "{}/{}_{}_hm.dat",
        prefix,
        meta.site.id,
        meta.model.to_uppercase()
    ));
    let f_hm = &mut File::create(&fname_hm)?;

    write_ensemble_data(&ens, f_ens)?;

    // Make a merged data and write that out too.
    let merged = ens.merge();

    write_merged_data(&merged, f_mrg)?;

    write_merged_heat_map_data(&mg, f_hm)?;

    Ok(())
}

/// Write the ensemble data in a gnuplot readable format.
fn write_ensemble_data<W: Write>(
    ens: &EnsembleSeries<AnalyzedData>,
    dest: &mut W,
) -> Result<(), Box<dyn Error>> {
    let EnsembleSeries { meta, data } = ens;

    // Write some comments about the meta data
    write_meta_data_header(meta, dest)?;
    // Write a header row
    writeln!(dest, "valid_time lead_time e0 de hdw")?;
    // Write out ensemble members/model runs in block format
    for (init_time, time_series) in data.iter() {
        writeln!(dest, "# init_time: {}", init_time.format(GP_DATE_FORMAT))?;
        for AnalyzedData {
            valid_time,
            lead_time,
            hdw,
            e0,
            de,
            ..
        } in time_series.as_ref().iter()
        {
            writeln!(
                dest,
                "{} {} {} {} {}",
                valid_time.format(GP_DATE_FORMAT),
                lead_time,
                e0.unpack(),
                de.unpack(),
                hdw
            )?;
        }

        // Block separator
        writeln!(dest)?;
    }
    Ok(())
}

/// Write the merged time series data in a gnuplot readable format
fn write_merged_data<W: Write>(
    mrg: &MergedSeries<AnalyzedData>,
    dest: &mut W,
) -> Result<(), Box<dyn Error>> {
    let MergedSeries { meta, data } = mrg;

    // Write some comments about the meta data
    write_meta_data_header(meta, dest)?;
    // Write a header row
    writeln!(dest, "valid_time lead_time dt0 e0 de hdw")?;
    // Write out ensemble members/model runs in block format

    for AnalyzedData {
        valid_time,
        lead_time,
        hdw,
        dt0,
        e0,
        de,
        ..
    } in data.as_ref().iter()
    {
        writeln!(
            dest,
            "{} {} {} {} {} {}",
            valid_time.format(GP_DATE_FORMAT),
            lead_time,
            dt0.unpack(),
            e0.unpack(),
            de.unpack(),
            hdw
        )?;
    }

    Ok(())
}

/// Write out the merged partiton data to a writer.
fn write_merged_heat_map_data<W: Write>(
    cape_parts: &MergedSeries<Vec<CapePartition>>,
    dest: &mut W,
) -> Result<(), Box<dyn Error>> {
    let MergedSeries { meta, data } = cape_parts;

    // Write some comments about the meta data
    write_meta_data_header(meta, dest)?;
    // Write a header row
    writeln!(dest, "valid_time dt dry_cape wet_cape")?;

    // Write out the data in x y z1 z2....
    for cp in data.as_ref().iter() {
        for CapePartition {
            valid_time,
            dt,
            dry,
            wet,
        } in cp.iter()
        {
            writeln!(
                dest,
                "{} {} {} {}",
                valid_time.format(GP_DATE_FORMAT),
                dt.unpack(),
                dry.unpack(),
                wet.unpack()
            )?
        }
        writeln!(dest)?;
    }
    Ok(())
}

/// Write a header to a data file/section in gnuplot comment form.
fn write_meta_data_header<W: Write>(meta: &MetaData, dest: &mut W) -> Result<(), Box<dyn Error>> {
    writeln!(
        dest,
        "# Site: {}\n# Model: {}\n# Start: {}\n# Now: {}\n# End: {}\n",
        meta.site.id,
        meta.model,
        meta.start.format(GP_DATE_FORMAT),
        meta.now.format(GP_DATE_FORMAT),
        meta.end.format(GP_DATE_FORMAT)
    )?;
    Ok(())
}

// TODO: make a function to plot the ensemble plumes
// TODO: make a function to plot the merged data and heat map together
// TODO: make a function to plot the de/e0 ratio vs hdw
