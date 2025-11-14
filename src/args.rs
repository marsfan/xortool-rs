use docopt::{ArgvMap, Docopt};

use crate::charset::get_charset;

pub struct Parameters {
    pub brute_chars: bool,
    pub brute_printable: bool,
    pub filename: String,
    pub filter_output: bool,
    pub frequency_spread: i32,
    pub input_is_hex: bool,
    pub known_key_length: Option<i32>,
    pub max_key_length: Option<i32>,
    pub most_frequent_char: Option<i32>,
    pub text_charset: Vec<u8>,
    pub known_plain: Vec<u8>,
    pub threshold: Option<i32>,
}

fn parse_char(ch: Option<&str>) -> Option<i32> {
    match ch {
        Some(mut c) => {
            if c.len() == 1 {
                return Some(c.bytes().collect::<Vec<u8>>()[0].into());
            }
            if c[0..2] == *"0x" || c[0..2] == *"\\x" {
                c = &c[2..];
            }
            if c.len() == 0 {
                panic!("Empty Char");
            }
            if c.len() > 2 {
                panic!("Char can be only a char letter or hex");
            }
            return Some(i32::from_str_radix(c, 16).unwrap());
        }
        None => None,
    }
}

fn parse_int(i: Option<&str>) -> Option<i32> {
    match i {
        Some(i) => Some(i.parse().unwrap()),
        None => None,
    }
}

fn get_option_string(arg: &str, parsed: &ArgvMap) -> Option<String> {
    let value = parsed.get_str(arg);
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn parse_optional_int(parsed: &ArgvMap, arg: &str) -> Option<i32> {
    let value = parsed.get_str(arg);
    if value.is_empty() {
        None
    } else {
        Some(value.parse().unwrap())
    }
}

pub fn parse_parameters(doc: &str, version: &str) -> Parameters {
    let p = Docopt::new(doc)
        .and_then(|dopt| dopt.version(Some(version.to_string())).parse())
        .unwrap_or_else(|e| e.exit());
    Parameters {
        brute_chars: p.get_bool("--brute-chars"),
        brute_printable: p.get_bool("--brute-printable"),
        filename: if p.get_str("FILE").is_empty() {
            "-".to_string()
        } else {
            p.get_str("FILE").to_string()
        },
        filter_output: p.get_bool("--filter-output"),
        frequency_spread: 0, // To be removed
        input_is_hex: p.get_bool("--hex"),
        known_key_length: parse_optional_int(&p, "--key-length"),
        max_key_length: parse_optional_int(&p, "--max-keylen"),
        most_frequent_char: parse_optional_int(&p, "--char"),
        text_charset: get_charset(p.get_str("--text-charset"))
            .unwrap()
            .as_bytes()
            .to_vec(),
        known_plain: p.get_str("--known-plaintext").bytes().collect(),
        threshold: parse_optional_int(&p, "--threshold"),
    }
}
