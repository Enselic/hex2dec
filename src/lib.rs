use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::{
    io::{self, BufRead, ErrorKind::InvalidInput}, 
    borrow::Cow, 
    num::ParseIntError
};

static REGEX: Lazy<Regex> = Lazy::new(|| 
    regex::Regex::new(r"\b(0x)?([0-9a-fA-F]{2,})\b").unwrap()
);
// Match at compile time
const NEWLINE: &str = if cfg!(windows) { "\r\n" } else { "\n" };

/// Error capturing implementation of [`regex::Regex::replace_all`](
/// https://docs.rs/regex/latest/regex/struct.Regex.html#method.replace_all).
/// Modified from [this](https://docs.rs/regex/latest/regex/struct.Regex.html#fallibility)
/// suggestion in [`regex`].
/// # Errors
/// This function will return an error when `replacement` fails on a capture.
/// # Examples
/// See examples in [`regex::Regex::replace_all`]. 
fn replace_all<'h, E>(
    re: &Regex,
    haystack: &'h str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
) -> Result<Cow<'h, str>, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(Cow::from(new))
}

/// Read and parse the standard input of the current process.
/// The given function `ok_callback` is applied to the result of [`hex2dec_line`] for each line.
/// Option to return early with [`Ok`] if a blank line (`"\r"` on Unix-like and `"\r\n"`
/// on Windows-like) is provided.
/// # Errors
/// This function errors if a line is failed to be read from STDIN (See 
/// [`io::BufRead::read_line`]) or if [`hex2dec_line`] fails on a line. 
/// If `skip_parse_errors` is set to `true` then `stop_on_error` will be ignored.
/// # Future
/// API may be changed so that this function returns any values produced by `f`.
/// Alternatively a secondary function may be provided.
pub fn parse_stdin<E: Fn(ParseIntError) -> io::Error, F: Fn(String)>(
    ok_callback: F, error_callback: E, 
    skip_parse_errors: bool, stop_on_error: bool, 
    break_nl: bool
) -> Result<(),io::Error>{
    let mut line = String::new();
    let mut handle = io::stdin().lock();

    loop {
        let nbytes = handle.read_line(&mut line)?;

        // Break if EOF
        // or a blank line is reached with break_newline true.
        if nbytes == 0 || (break_nl && (line == NEWLINE)) { return Ok(()); }

        match hex2dec_line(&line, skip_parse_errors) {
            Ok(s) => ok_callback(s),
            Err(e) => if stop_on_error { return Err(error_callback(e))} 
            else { error_callback(e); }
        };

        line.clear();
    }
}

/// Wrapper for [`parse_stdin`] for usage with CI utilities.
/// Errors are redirected to STDERR for compatibility with CI operations.
pub fn parse_ci<E: Fn(ParseIntError) -> io::Error, F: Fn(String)>(
    ok_callback: F, error_callback: E, 
    skip_parse_errors: bool, stop_on_error: bool,
    break_nl: bool
){
    if let Err(e) = parse_stdin(
        ok_callback, error_callback, skip_parse_errors, stop_on_error, break_nl
    ) { 
        match e.kind() {
            InvalidInput => eprint!("An error occured in hex2dec_line. {}", e),
            _ => eprint!("An error occured in parse_stdin. {}", e),
        }
    }
}

/// Convert values within a string from hex to decimal notation.
/// # Errors
/// This function errors when the program fails to parse any hex value contained
/// within the supplied string. If skip_error is set to `true` then parsing errors will
/// be ignored and the matched substring will remain in place.
/// # Examples
/// ```rust
/// use hex2dec::hex2dec_line;
/// let s = hex2dec_line("0x5a200", false);
/// assert_eq!(s.unwrap(), " 369152"); // String is padded to the same length
/// ``` 
pub fn hex2dec_line<S: AsRef<str>>(line: S, skip_error: bool)
 -> Result<String, std::num::ParseIntError>{
    let new_str = replace_all(
        &REGEX, line.as_ref(),
        |caps: &Captures| {
            let m = caps.get(0).unwrap();
            let format_length = m.len();
            let hex_str = caps.get(2).unwrap().as_str();

            let dec = if skip_error {
                match u128::from_str_radix(hex_str, 16) {
                    Ok(d) => d,
                    Err(_) => return Ok(m.as_str().to_owned()) // Use original substring
                }
            } else {
                u128::from_str_radix(hex_str, 16)?
            };
            
            Ok(
                format!(
                    "{:width$}",
                    dec,
                    width = format_length
                )
            )
        })?
        .into_owned();

    Ok(new_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex2dec_line(){
        let tests = [
            ("", ""),
            (" a ", " a "),
            (" 1 ", " 1 "),
            ("0x1", "0x1"),
            ("0x12", "  18"),
            ("  0x1  ", "  0x1  "),
            ("  0x1  ", "  0x1  "),
            (
                "  Magic:   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00 ",
                "  Magic:   127 69 76 70  2  1  1  0  0  0  0  0  0  0  0  0 ",
            ),
            (
                "        Entry point address:               0x5a200",
                "        Entry point address:                369152",
            ),
        ];
        for test in tests {
            assert_eq!(hex2dec_line(test.0, false).unwrap(), test.1);
        }
    }
    
    // #[test]
    // fn stdin() -> Result<(), io::Error>{
    //     parse_stdin(|s| println!("{}", s))?;
    //     Ok(())
    // }
}
