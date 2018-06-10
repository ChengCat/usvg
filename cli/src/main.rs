extern crate usvg;
extern crate fern;
extern crate log;
#[macro_use] extern crate gumdrop;

use std::fmt;
use std::fs::File;
use std::io::{ self, Read, Write };
use std::path::Path;
use std::process;
use std::str::FromStr;

use gumdrop::Options;

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


#[derive(Clone, Copy, Debug)]
struct Dpi(u32);

impl FromStr for Dpi {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: u32 = s.parse().map_err(|_| "invalid number")?;

        if n >= 10 && n <= 4000 {
            Ok(Dpi(n))
        } else {
            Err("DPI out of bounds")
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct Indent(svgdom::Indent);

impl FromStr for Indent {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let indent = match s {
            "none" => svgdom::Indent::None,
            "0" => svgdom::Indent::Spaces(0),
            "1" => svgdom::Indent::Spaces(1),
            "2" => svgdom::Indent::Spaces(2),
            "3" => svgdom::Indent::Spaces(3),
            "4" => svgdom::Indent::Spaces(4),
            "tabs" => svgdom::Indent::Tabs,
            _ => return Err("invalid INDENT value"),
        };

        Ok(Indent(indent))
    }
}


#[derive(Debug, Options)]
struct Args {
    #[options(help = "Prints help information")]
    help: bool,

    #[options(short = "V", help = "Prints version information")]
    version: bool,

    #[options(short = "c", no_long, help = "Prints the output SVG to the stdout")]
    stdout: bool,

    #[options(help = "Keeps groups with non-empty ID")]
    keep_named_groups: bool,

    #[options(no_short, help = "Sets the resolution", meta = "DPI")]
    dpi: Dpi,

    #[options(no_short, help = "Sets the XML nodes indent", meta = "INDENT")]
    indent: Indent,

    #[options(no_short, help = "Sets the XML attributes indent", meta = "INDENT")]
    attrs_indent: Indent,

    #[options(free)]
    free: Vec<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            help: false,
            version: false,
            stdout: false,
            keep_named_groups: false,
            dpi: Dpi(96),
            indent: Indent(svgdom::Indent::Spaces(4)),
            attrs_indent: Indent(svgdom::Indent::None),
            free: Vec::new(),
        }
    }
}


fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    let args = match Args::parse_args_default(&args[1..]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            process::exit(1);
        }
    };

    if args.help {
        print_help();
        process::exit(0);
    }

    if args.version {
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

fn print_help() {
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
        --dpi DPI               Sets the resolution
                                [default: 96] [possible values: 10..4000]
        --indent INDENT         Sets the XML nodes indent
                                [values: none, 0, 1, 2, 3, 4, tabs] [default: 4]
        --attrs-indent INDENT   Sets the XML attributes indent
                                [values: none, 0, 1, 2, 3, 4, tabs] [default: none]

ARGS:
    <in-svg>                    Input file
    <out-svg>                   Output file
");
}

fn process(args: &Args) -> Result<(), String> {
    if args.free.is_empty() {
        return Err(format!("no positional arguments are provided"));
    }

    let (in_svg, out_svg) = {
        let in_svg = &args.free[0];
        let out_svg = args.free.get(1);
        let out_svg = out_svg.map(String::as_ref);

        let svg_from = if in_svg == "-" && args.stdout {
            InputFrom::Stdin
        } else if let Some("-") = out_svg {
            InputFrom::Stdin
        } else {
            InputFrom::File(in_svg)
        };

        let svg_to = if args.stdout {
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
            InputFrom::File(ref f) => Some(f.into()),
        },
        dpi: args.dpi.0 as f64,
        keep_named_groups: args.keep_named_groups,
    };

    let input_str = match in_svg {
        InputFrom::Stdin => load_stdin(),
        InputFrom::File(ref path) => {
            usvg::load_svg_file(Path::new(path)).map_err(|e| e.to_string())
        }
    }?;

    let tree = usvg::Tree::from_str(&input_str, &re_opt)
                    .map_err(|e| format!("{}", e))?;

    let dom_opt = svgdom::WriteOptions {
        indent: args.indent.0,
        attributes_indent: args.attrs_indent.0,
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
