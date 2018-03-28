#[macro_use] extern crate clap;
#[macro_use] extern crate derive_error;
extern crate usvg;
extern crate svgdom;
extern crate fern;
extern crate log;


use std::io::{ self, Read, Write };
use std::fs::File;
use std::fmt;

use clap::{ App, Arg, ArgMatches };

use usvg::tree::prelude::*;

use svgdom::{ WriteBuffer, ChainedErrorExt };


fn main() {
    let args = App::new("usvg")
        .version(env!("CARGO_PKG_VERSION"))
        .about("usvg (micro SVG) is an SVG simplification tool")
        .usage("usvg [FLAGS] [OPTIONS] <in-svg> <out-svg> # from file to file\n    \
                usvg [FLAGS] [OPTIONS] -c <in-svg>        # from file to stdout\n    \
                usvg [FLAGS] [OPTIONS] <out-svg> -        # from stdin to file\n    \
                usvg [FLAGS] [OPTIONS] -c -               # from stdin to stdout")
        .arg(Arg::with_name("in-svg")
            .help("Input file")
            .required(true)
            .index(1)
            .validator(is_svg))
        .arg(Arg::with_name("out-svg")
            .help("Output file")
            .required_unless("stdout")
            .index(2)
            .validator(is_svg))
        .arg(Arg::with_name("stdout")
            .short("c")
            .help("Prints the output SVG to the stdout"))
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

    if let Err(e) = process(&args) {
        match e {
            Error::Usvg(ref e) => eprintln!("{}.", e.full_chain()),
            Error::Io(ref e) => eprintln!("Error: {}.", e),
        }

        std::process::exit(1);
    }
}

fn is_svg(val: String) -> Result<(), String> {
    let val = val.to_lowercase();
    if val.ends_with(".svg") || val.ends_with(".svgz") || val == "-" {
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

#[derive(Error, Debug)]
enum Error {
    Usvg(usvg::Error),
    Io(io::Error),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum InputFrom<'a> {
    Stdin,
    File(&'a str),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum OutputTo<'a> {
    Stdout,
    File(&'a str),
}

fn process(args: &ArgMatches) -> Result<(), Error> {
    let (in_svg, out_svg) = {
        let in_svg = args.value_of("in-svg").unwrap();
        let out_svg = args.value_of("out-svg");

        let svg_from = if in_svg == "-" && args.is_present("stdout") {
            InputFrom::Stdin
        } else if let Some("-") = out_svg {
            InputFrom::Stdin
        } else {
            InputFrom::File(in_svg)
        };

        let svg_to = if args.is_present("stdout") {
            OutputTo::Stdout
        } else if let Some("-") = out_svg {
            OutputTo::File(in_svg)
        } else {
            OutputTo::File(out_svg.unwrap())
        };

        (svg_from, svg_to)
    };

    let re_opt = usvg::Options {
        path: match in_svg {
            InputFrom::Stdin => None,
            InputFrom::File(f) => Some(f.into()),
        },
        dpi: value_t!(args.value_of("dpi"), u32).unwrap() as f64,
        keep_named_groups: args.is_present("keep-named-groups"),
    };

    fern::Dispatch::new()
        .format(log_format)
        .level(log::LevelFilter::Warn)
        .chain(std::io::stderr())
        .apply().unwrap();

    let tree = match in_svg {
        InputFrom::Stdin => {
            let s = load_stdin()?;
            usvg::parse_tree_from_data(&s, &re_opt)
        }
        InputFrom::File(path) => usvg::parse_tree_from_file(path, &re_opt),
    }?;

    let dom_opt = svgdom::WriteOptions {
        indent: svgdom::Indent::Spaces(2),
        attributes_indent: svgdom::Indent::Spaces(3),
        attributes_order: svgdom::AttributesOrder::Specification,
        .. svgdom::WriteOptions::default()
    };

    let doc = tree.to_svgdom();

    let mut output_data = Vec::new();
    doc.write_buf_opt(&dom_opt, &mut output_data);

    match out_svg {
        OutputTo::Stdout => {
            io::stdout().write_all(&output_data)?;
        }
        OutputTo::File(path) => {
            let mut f = File::create(path)?;
            f.write_all(&output_data)?;
        }
    }

    Ok(())
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

fn load_stdin() -> Result<String, io::Error> {
    let mut s = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    handle.read_to_string(&mut s)?;

    Ok(s)
}
