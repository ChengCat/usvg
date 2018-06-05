extern crate usvg;
extern crate fern;
extern crate log;
extern crate getopts;

use std::fmt;
use std::fs::File;
use std::io::{ self, Read, Write };
use std::path::Path;
use std::process;
use std::str::FromStr;

use getopts::Matches;

use usvg::svgdom;

use svgdom::WriteBuffer;


#[derive(Clone, PartialEq, Debug)]
enum InputFrom<'a> {
    Stdin,
    File(&'a str),
}

#[derive(Clone, PartialEq, Debug)]
enum OutputTo<'a> {
    Stdout,
    File(&'a str),
}


fn main() {
    let args: Vec<String> = ::std::env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "");
    opts.optflag("V", "version", "");
    opts.optflag("c", "", "");
    opts.optflag("", "keep-named-groups", "");
    opts.optopt("", "dpi", "", "");
    opts.optopt("", "indent", "", "");
    opts.optopt("", "attrs-indent", "", "");

    let args = match opts.parse(&args[1..]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}.", e);
            process::exit(0);
        }
    };

    if args.opt_present("help") {
        print_help();
        process::exit(0);
    }

    if args.opt_present("version") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    fern::Dispatch::new()
        .format(log_format)
        .level(log::LevelFilter::Warn)
        .chain(std::io::stderr())
        .apply().unwrap();

    if let Err(e) = process(&args) {
        eprintln!("Error: {}.", e.to_string());
        std::process::exit(1);
    }
}

pub fn print_help() {
    print!("\
usvg (micro SVG) is an SVG simplification tool.

USAGE:
    usvg [OPTIONS] <in-svg> <out-svg> # from file to file
    usvg [OPTIONS] -c <in-svg>        # from file to stdout
    usvg [OPTIONS] <out-svg> -        # from stdin to file
    usvg [OPTIONS] -c -               # from stdin to stdout

OPTIONS:
    -h, --help                  Prints help information
    -V, --version               Prints version information
    -c                          Prints the output SVG to the stdout
        --keep-named-groups     Keeps groups with non-empty ID
        --dpi=<DPI>             Sets the resolution
                                [default: 96] [possible values: 10..4000]
        --indent=<INDENT>       Sets the XML nodes indent
                                [values: none, 0, 1, 2, 3, 4, tabs] [default: 4]
        --attrs-indent=<INDENT> Sets the XML attributes indent
                                [values: none, 0, 1, 2, 3, 4, tabs] [default: none]

ARGS:
    <in-svg>                    Input file
    <out-svg>                   Output file
");
}

fn process(args: &Matches) -> Result<(), String> {
    let (in_svg, out_svg) = {
        let in_svg = &args.free[0];
        let out_svg = args.free.get(1);
        let out_svg = out_svg.map(String::as_ref);

        let svg_from = if in_svg == "-" && args.opt_present("c") {
            InputFrom::Stdin
        } else if let Some("-") = out_svg {
            InputFrom::Stdin
        } else {
            InputFrom::File(in_svg)
        };

        let svg_to = if args.opt_present("c") {
            OutputTo::Stdout
        } else if let Some("-") = out_svg {
            OutputTo::File(in_svg)
        } else {
            OutputTo::File(out_svg.unwrap())
        };

        (svg_from, svg_to)
    };

    let dpi = get_type(&args, "dpi", "DPI")?.unwrap_or(96);
    if dpi < 10 || dpi > 4000 {
        return Err(format!("DPI out of bounds"));
    }

    let re_opt = usvg::Options {
        path: match in_svg {
            InputFrom::Stdin => None,
            InputFrom::File(ref f) => Some(f.into()),
        },
        dpi: dpi as f64,
        keep_named_groups: args.opt_present("keep-named-groups"),
    };

    let input_str = match in_svg {
        InputFrom::Stdin => load_stdin(),
        InputFrom::File(ref path) => {
            usvg::load_svg_file(Path::new(path)).map_err(|e| e.to_string())
        }
    }?;

    let tree = usvg::Tree::from_str(&input_str, &re_opt);

    let dom_opt = svgdom::WriteOptions {
        indent: get_indent(args, "indent", svgdom::Indent::Spaces(4))?,
        attributes_indent: get_indent(args, "attrs-indent", svgdom::Indent::None)?,
        attributes_order: svgdom::AttributesOrder::Specification,
        .. svgdom::WriteOptions::default()
    };

    let doc = tree.to_svgdom();

    let mut output_data = Vec::new();
    doc.write_buf_opt(&dom_opt, &mut output_data);

    match out_svg {
        OutputTo::Stdout => {
            io::stdout().write_all(&output_data)
                .map_err(|_| format!("failed to write to the stdout"))?;
        }
        OutputTo::File(path) => {
            let mut f = File::create(path)
                .map_err(|_| format!("failed to create the output file"))?;
            f.write_all(&output_data)
                .map_err(|_| format!("failed to write to the output file"))?;
        }
    }

    Ok(())
}

fn get_type<T: FromStr>(args: &getopts::Matches, name: &str, type_name: &str)
    -> Result<Option<T>, String>
{
    match args.opt_str(name) {
        Some(v) => {
            let t = v.parse().map_err(|_| format!("invalid {}: '{}'", type_name, v))?;
            Ok(Some(t))
        }
        None => Ok(None),
    }
}

fn get_indent(args: &getopts::Matches, name: &str, def: svgdom::Indent)
    -> Result<svgdom::Indent, String>
{
    match args.opt_str(name) {
        Some(v) => {
            match v.as_str() {
                "none" => Ok(svgdom::Indent::None),
                "0" => Ok(svgdom::Indent::Spaces(0)),
                "1" => Ok(svgdom::Indent::Spaces(1)),
                "2" => Ok(svgdom::Indent::Spaces(2)),
                "3" => Ok(svgdom::Indent::Spaces(3)),
                "4" => Ok(svgdom::Indent::Spaces(4)),
                "tabs" => Ok(svgdom::Indent::Tabs),
                _ => Err(format!("invalid indent value")),
            }
        }
        None => Ok(def),
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

fn load_stdin() -> Result<String, String> {
    let mut s = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    handle.read_to_string(&mut s)
          .map_err(|_| format!("provided data has not an UTF-8 encoding"))?;

    Ok(s)
}
