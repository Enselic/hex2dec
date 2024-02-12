use hex2dec::parse_ci;
use std::{io, process};

fn main() {
    let callback = |s| print!("{}", s);
    let on_error = |e| io::Error::new(io::ErrorKind::InvalidInput, e);
    // (bool, bool, bool) <-> (skip_parse_errors, stop_on_error, break_nl)
    parse_ci(callback, on_error, false, true, false);
    process::exit(0);
}
