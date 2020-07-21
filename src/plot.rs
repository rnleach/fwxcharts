//! Functions used for plotting data and producing output.
use crate::{
    messages::{InnerMessage, Message},
    timeseries::{EnsembleSeries, MergedSeries, MetaData},
    types::{parse_sounding, AnalyzedData},
};
use bufcli::{ClimoElement, ClimoQueryInterface, Percentile};
use crossbeam::{crossbeam_channel::unbounded, scope};
use metfor::Quantity;
use rayon::iter::{IterBridge, ParallelBridge, ParallelIterator};
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
pub fn plot_all<I>(iter: I, prefix: &str, mut climo: Option<ClimoQueryInterface>)
where
    I: Iterator<Item = Message> + ParallelBridge + Send,
    IterBridge<I>: ParallelIterator<Item = Message> + Send,
{
    let (plot_sender, plot_receiver) = unbounded();

    scope(|s| {
        s.spawn(move |_| {
            iter.par_bridge()
                .filter_map(|msg| match msg.payload() {
                    InnerMessage::StringData(ens_list_strings) => {
                        let start = ens_list_strings.meta.start;
                        let end = ens_list_strings.meta.end;
                        let ens_ser_anal = ens_list_strings
                            .filter_map(|str_data| parse_sounding(str_data, start, end));

                        if ens_ser_anal.is_empty() {
                            None
                        } else {
                            Some(ens_ser_anal)
                        }
                    }
                    InnerMessage::BufkitDataError(err) => {
                        println!("Error: {:?}", err);
                        None
                    }
                })
                .map(|ens_ser_anal| ens_ser_anal.filter_map_inner(AnalyzedData::analyze))
                .for_each(|analyzed_data| plot_sender.send(analyzed_data).unwrap());
        });

        let gp_in = &mut launch_gnuplot(prefix).unwrap();
        for analyzed_data in plot_receiver {
            gp_plot_ens(gp_in, &analyzed_data).unwrap_or_else(|err| println!("{:?}", err));
            let merged = analyzed_data.merge();
            gp_plot_mrg(gp_in, &merged, climo.as_mut()).unwrap_or_else(|err| println!("{:?}", err));
        }
    })
    .unwrap();
}

/// Given an iterator over `StringData` loaded from Bufkit files, filter out any failed results
/// and save the data in files suitable for gnuplot.
///
/// # Arguments
/// iter - an iterator over ensembles of model runs, make the plot and save it for each ensemble.
/// prefix - The path to the folder where you want the plots saved.
pub fn save_all(
    iter: impl Iterator<Item = Message>,
    prefix: &str,
    mut climo: Option<ClimoQueryInterface>,
) -> Result<(), Box<dyn Error>> {
    use InnerMessage::*;

    iter.filter_map(|msg| match msg.payload() {
        StringData(ens_list_strings) => {
            let start = ens_list_strings.meta.start;
            let end = ens_list_strings.meta.end;
            let ens_ser_anal =
                ens_list_strings.filter_map(|str_data| parse_sounding(str_data, start, end));

            if ens_ser_anal.is_empty() {
                None
            } else {
                Some(ens_ser_anal)
            }
        }
        BufkitDataError(err) => {
            println!("Error: {:?}", err);
            None
        }
    })
    .map(|ens_ser_anal| ens_ser_anal.filter_map_inner(AnalyzedData::analyze))
    .for_each(|analyzed_data| gp_save(prefix, analyzed_data, climo.as_mut()).unwrap_or(()));

    Ok(())
}

const GP_INIT: &str = include_str!("plot/initialize.plt");
const GP_PLOT_ENS: &str = include_str!("plot/ens_template.plt");
const GP_PLOT_MRG: &str = include_str!("plot/mrg_template.plt");
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
    mut climo: Option<&mut ClimoQueryInterface>,
) -> Result<(), Box<dyn Error>> {
    let MergedSeries::<AnalyzedData> { meta: meta_mg, .. } = &mg;

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
        meta_mg.site.description(),
        meta_mg.model.to_uppercase()
    )?;
    writeln!(
        gp,
        "output_name=\"{}_{}\"",
        meta_mg.site.station_num,
        meta_mg.model.to_uppercase()
    )?;

    writeln!(gp, "$data << EOD")?;
    write_merged_data(mg, gp)?;
    writeln!(gp, "EOD")?;

    // Try to get the climate data for the HDW and add that to the data
    writeln!(gp, "$hdw_climo << EOD")?;
    write_climo(&meta_mg, ClimoElement::HDW, gp, &mut climo)?;
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
        meta.site.name.as_ref().unwrap_or(&meta.site.description()),
        meta.model.to_uppercase()
    )?;
    writeln!(
        gp,
        "output_name=\"{}_{}_ens.png\"",
        meta.site.station_num,
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

/// Save a set of data
fn gp_save(
    prefix: &str,
    ens: EnsembleSeries<AnalyzedData>,
    mut climo: Option<&mut ClimoQueryInterface>,
) -> Result<(), Box<dyn Error>> {
    let EnsembleSeries::<AnalyzedData> { meta, .. } = &ens;

    // Build the file names to save the data to
    let fname_ens: PathBuf = PathBuf::from(&format!(
        "{}/{}_{}_ens.dat",
        prefix,
        meta.site.station_num,
        meta.model.to_uppercase()
    ));
    let f_ens = &mut File::create(&fname_ens)?;
    let fname_mrg: PathBuf = PathBuf::from(&format!(
        "{}/{}_{}_mrg.dat",
        prefix,
        meta.site.station_num,
        meta.model.to_uppercase()
    ));
    let f_mrg = &mut File::create(&fname_mrg)?;

    let fname_cli: PathBuf = PathBuf::from(&format!(
        "{}/{}_{}_cli.dat",
        prefix,
        meta.site.station_num,
        meta.model.to_uppercase()
    ));
    let f_cli = &mut File::create(&fname_cli)?;

    write_ensemble_data(&ens, f_ens)?;

    // Make a merged data and write that out too.
    let merged = ens.merge();

    write_merged_data(&merged, f_mrg)?;

    write_climo(&merged.meta, ClimoElement::HDW, f_cli, &mut climo)?;

    Ok(())
}

/// Write the ensemble data in a gnuplot readable format.
fn write_ensemble_data<W: Write>(
    ens: &EnsembleSeries<AnalyzedData>,
    dest: &mut W,
) -> Result<(), Box<dyn Error>> {
    let EnsembleSeries { meta, data } = ens;

    // Write some comments about the meta data
    write_meta_data_header(&meta, dest)?;
    // Write a header row
    writeln!(dest, "valid_time lead_time blow_up_dt blow_up_height hdw")?;
    // Write out ensemble members/model runs in block format
    for (init_time, time_series) in data.iter() {
        writeln!(dest, "# init_time: {}", init_time.format(GP_DATE_FORMAT))?;
        for AnalyzedData {
            valid_time,
            lead_time,
            hdw,
            blow_up_dt,
            blow_up_height,
        } in time_series.as_ref().iter()
        {
            writeln!(
                dest,
                "{} {} {} {} {}",
                valid_time.format(GP_DATE_FORMAT),
                lead_time,
                blow_up_dt.unpack(),
                blow_up_height.unpack(),
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
    write_meta_data_header(&meta, dest)?;
    // Write a header row
    writeln!(dest, "valid_time lead_time blow_up_dt blow_up_height hdw")?;
    // Write out ensemble members/model runs in block format

    for AnalyzedData {
        valid_time,
        lead_time,
        hdw,
        blow_up_dt,
        blow_up_height,
    } in data.as_ref().iter()
    {
        writeln!(
            dest,
            "{} {} {} {} {}",
            valid_time.format(GP_DATE_FORMAT),
            lead_time,
            blow_up_dt.unpack(),
            blow_up_height.unpack(),
            hdw
        )?;
    }

    Ok(())
}

/// Write out the climate data for the HDW
fn write_climo<W: Write>(
    meta: &MetaData,
    element: ClimoElement,
    dest: &mut W,
    climo: &mut Option<&mut ClimoQueryInterface>,
) -> Result<(), Box<dyn Error>> {
    write_meta_data_header(meta, dest)?;

    let MetaData {
        site,
        model,
        start,
        end,
        ..
    } = meta;

    writeln!(
        dest,
        "valid_time min 10th 20th 30th 40th median 60th 70th 80th 90th max"
    )?;

    if let Some(hourly_deciles) = climo.as_mut().and_then(|climo_iface| {
        climo_iface
            .hourly_deciles(site, model, element, *start, *end)
            .ok()
    }) {
        for (vt, deciles) in hourly_deciles {
            writeln!(
                dest,
                "{} {} {} {} {} {} {} {} {} {} {} {}",
                vt.format(GP_DATE_FORMAT),
                deciles.value_at_percentile(Percentile::from(0)),
                deciles.value_at_percentile(Percentile::from(10)),
                deciles.value_at_percentile(Percentile::from(20)),
                deciles.value_at_percentile(Percentile::from(30)),
                deciles.value_at_percentile(Percentile::from(40)),
                deciles.value_at_percentile(Percentile::from(50)),
                deciles.value_at_percentile(Percentile::from(60)),
                deciles.value_at_percentile(Percentile::from(70)),
                deciles.value_at_percentile(Percentile::from(80)),
                deciles.value_at_percentile(Percentile::from(90)),
                deciles.value_at_percentile(Percentile::from(100)),
            )?;
        }
    } else {
        writeln!(
            dest,
            "{} NaN NaN NaN NaN NaN NaN NaN NaN NaN NaN NaN",
            start.format(GP_DATE_FORMAT),
        )?;
    }

    Ok(())
}

/// Write a header to a data file/section in gnuplot comment form.
fn write_meta_data_header<W: Write>(meta: &MetaData, dest: &mut W) -> Result<(), Box<dyn Error>> {
    writeln!(
        dest,
        "# Site: {}\n# Model: {}\n# Start: {}\n# Now: {}\n# End: {}\n",
        meta.site.description(),
        meta.model,
        meta.start.format(GP_DATE_FORMAT),
        meta.now.format(GP_DATE_FORMAT),
        meta.end.format(GP_DATE_FORMAT)
    )?;
    Ok(())
}
