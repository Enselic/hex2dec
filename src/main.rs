use once_cell::sync::Lazy;
use regex::{Captures, Regex};

static REGEX: Lazy<Regex> = Lazy::new(|| regex::Regex::new(r"\b(0x)?([0-9a-fA-F]{2,})\b").unwrap());

fn main() {
    for line in std::io::stdin().lines() {
        let line = line.unwrap();
        println!("{}", hex2dec_line(&line));
    }
}

fn hex2dec_line(line: &str) -> String {
    REGEX
        .replace_all(line, |caps: &Captures| {
            let m = caps.get(0).unwrap();
            let hex = caps.get(2).unwrap().as_str();
            format!(
                "{:width$}",
                u128::from_str_radix(hex, 16).unwrap(),
                width = m.len()
            )
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex2dec_line() {
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
            assert_eq!(hex2dec_line(test.0), test.1);
        }
    }
}
