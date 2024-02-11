use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::{io::BufRead, borrow::Cow};

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
/// See [`regex::Regex::replace_all`] as before. 
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
/// The given function `f` is applied to the result of [`hex2dec_line`] for each line.
/// ### Note
/// Currently if a blank line (`"\r"` on Unix-like and `"\r\n"` on Windows-like) is provided
/// then the function will return early with an [`Ok`] value.
/// # Errors
/// This function errors if a line is failed to be read from STDIN (See [`std::io::BufRead::read_line`])
/// or if [`hex2dec_line`] fails on a line.
/// # Future
/// API may be changed so that this function returns any values produced by `f`.
/// Alternatively a secondary function may be provided.
pub fn parse_stdin<F: Fn(String)>(f: F) -> Result<(),std::io::Error>{
    let mut line = String::new();
    let mut handle = std::io::stdin().lock();

    loop {
        let nbytes = handle.read_line(&mut line)?;

        // Potentially change this.
        // Doesn't allow for blank lines in STDIN
        if nbytes == 0 || line == NEWLINE { return Ok(()); }

        match hex2dec_line(&line) {
            Ok(s) => f(s),
            Err(_) => return Err(std::io::ErrorKind::InvalidInput.into())
        };

        line.clear();
    }
}

/// Convert values within a string from hex to decimal notation.
/// # Errors
/// This function errors when the program fails to parse any hex value contained within the supplied string.
/// # Examples
/// ```rust
/// use hex2dec::hex2dec_line;
/// let s = hex2dec_line("0x5a200");
/// assert_eq!(s.unwrap(), " 369152"); // String is padded to the same length
/// ``` 
pub fn hex2dec_line<S: AsRef<str>>(line: S) -> Result<String, std::num::ParseIntError>{
    let new_str = replace_all(
        &REGEX, line.as_ref(),
        |caps: &Captures| {
            let format_length = caps.get(0).unwrap().len();
            let hex_str = caps.get(2).unwrap().as_str();

            let dec = u128::from_str_radix(hex_str, 16)?;
            
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
            assert_eq!(hex2dec_line(test.0).unwrap(), test.1);
        }
    }
    
    // #[test]
    // fn stdin() -> Result<(), std::io::Error>{
    //     parse_stdin(|s| println!("{}", s))?;
    //     Ok(())
    // }
}
