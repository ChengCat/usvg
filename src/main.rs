#[macro_use] extern crate clap;
extern crate usvg;
extern crate svgdom;
extern crate fern;
extern crate log;


use std::io::Write;
use std::fs::File;
use std::fmt;

use clap::{ App, Arg };

use usvg::tree::prelude::*;

use svgdom::WriteBuffer;


fn main() {
    let args = App::new("usvg")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("in-svg")
            .help("Input file")
            .required(true)
            .index(1)
            .validator(is_svg))
        .arg(Arg::with_name("out-svg")
            .help("Output file")
            .index(2)
            .validator(is_svg))
        .arg(Arg::with_name("dpi")
            .long("dpi")
            .help("Sets the resolution [72..4000]")
            .value_name("DPI")
            .default_value("96")
            .validator(is_dpi))
        .arg(Arg::with_name("keep-named-groups")
            .long("keep-named-groups")
            .help("Keeps groups with non-empty ID"))
        .get_matches();

    let in_svg  = args.value_of("in-svg").unwrap();
    let out_png = args.value_of("out-svg").unwrap();

    let re_opt = usvg::Options {
        path: Some(in_svg.into()),
        dpi: value_t!(args.value_of("dpi"), u32).unwrap() as f64,
        keep_named_groups: args.is_present("keep-named-groups"),
    };

    fern::Dispatch::new()
        .format(log_format)
        .level(log::LevelFilter::Warn)
        .chain(std::io::stderr())
        .apply().unwrap();

    let tree = usvg::parse_tree_from_file(in_svg, &re_opt).unwrap();

    let dom_opt = svgdom::WriteOptions {
        indent: svgdom::Indent::Spaces(2),
        attributes_indent: svgdom::Indent::Spaces(3),
        attributes_order: svgdom::AttributesOrder::Specification,
        .. svgdom::WriteOptions::default()
    };

    let doc = tree.to_svgdom();

    let mut output_data = Vec::new();
    doc.write_buf_opt(&dom_opt, &mut output_data);

    let mut f = File::create(out_png).unwrap();
    f.write_all(&output_data).unwrap();
}

fn is_svg(val: String) -> Result<(), String> {
    let val = val.to_lowercase();
    if val.ends_with(".svg") || val.ends_with(".svgz") {
        Ok(())
    } else {
        Err(String::from("The input file format must be SVG(Z)."))
    }
}

fn is_dpi(val: String) -> Result<(), String> {
    let n = match val.parse::<u32>() {
        Ok(v) => v,
        Err(e) => return Err(format!("{}", e)),
    };

    if n >= 72 && n <= 4000 {
        Ok(())
    } else {
        Err(String::from("Invalid DPI value."))
    }
}

fn log_format(
    out: fern::FormatCallback,
    message: &fmt::Arguments,
    record: &log::Record,
) {
    let lvl = match record.level() {
        log::Level::Error => "Error",
        log::Level::Warn  => "Warning",
        log::Level::Info  => "Info",
        log::Level::Debug => "Debug",
        log::Level::Trace => "Trace",
    };

    out.finish(format_args!(
        "{} (in {}:{}): {}",
        lvl,
        record.target(),
        record.line().unwrap_or(0),
        message
    ))
}
