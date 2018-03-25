extern crate assert_cli;
extern crate tempdir;
#[macro_use] extern crate pretty_assertions;

use std::fmt;

use tempdir::TempDir;

const APP_PATH: &str = "target/debug/usvg";

#[test]
fn test1() {
    let dir = TempDir::new("usvg").unwrap();
    let file_out = dir.path().join("test1.svg");
    let file_out = file_out.to_str().unwrap();

    let args = &[
        APP_PATH,
        "tests/images/test1-in.svg",
        file_out,
    ];

    assert_cli::Assert::command(args)
        .stdout().is("")
        .stderr().is("")
        .unwrap();

    cmp_files("tests/images/test1-out.svg", file_out);
}

#[derive(Clone, Copy, PartialEq)]
struct MStr<'a>(&'a str);

impl<'a> fmt::Debug for MStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn cmp_files(path1: &str, path2: &str) {
    assert_eq!(MStr(&load_file(path1)), MStr(&load_file(path2)));
}

fn load_file(path: &str) -> String {
    use std::fs;
    use std::io::Read;

    let mut file = fs::File::open(path).unwrap();
    let length = file.metadata().unwrap().len() as usize;

    let mut s = String::with_capacity(length + 1);
    file.read_to_string(&mut s).unwrap();

    s
}
