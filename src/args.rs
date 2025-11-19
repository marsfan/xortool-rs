/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use docopt::{ArgvMap, Docopt};

use crate::{charset::get_charset, error::XorError};

pub struct Parameters {
    pub brute_chars: bool,
    pub brute_printable: bool,
    pub filename: String,
    pub filter_output: bool,
    pub input_is_hex: bool,
    pub known_key_length: Option<i32>,
    pub max_key_length: Option<i32>,
    pub most_frequent_char: Option<u8>,
    pub text_charset: Vec<u8>,
    pub known_plain: Vec<u8>,
    pub threshold: Option<i32>,
}

fn parse_char(parsed: &ArgvMap, arg: &str) -> Option<u8> {
    let mut ch = parsed.get_str(arg);
    if ch.is_empty() {
        None
    } else {
        if ch.len() == 1 {
            return Some(ch.bytes().collect::<Vec<u8>>()[0].into());
        }
        if ch[0..2] == *"0x" || ch[0..2] == *"\\x" {
            ch = &ch[2..];
        }
        if ch.len() == 0 {
            panic!("Empty Char");
        }
        if ch.len() > 2 {
            panic!("Char can be only a char letter or hex");
        }
        return Some(u8::from_str_radix(ch, 16).unwrap());
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

pub fn parse_parameters(
    doc: &str,
    version: &str,
    args: Option<Vec<String>>,
) -> Result<Parameters, XorError> {
    let args = match args {
        Some(a) => a,
        None => std::env::args().collect(),
    };
    let p = Docopt::new(doc)
        .and_then(|dopt| dopt.version(Some(version.to_string())).argv(args).parse());
    match p {
        Ok(p) => Ok(Parameters {
            brute_chars: p.get_bool("--brute-chars"),
            brute_printable: p.get_bool("--brute-printable"),
            filename: if p.get_str("FILE").is_empty() {
                "-".to_string()
            } else {
                p.get_str("FILE").to_string()
            },
            filter_output: p.get_bool("--filter-output"),
            input_is_hex: p.get_bool("--hex"),
            known_key_length: parse_optional_int(&p, "--key-length"),
            max_key_length: parse_optional_int(&p, "--max-keylen"),
            most_frequent_char: parse_char(&p, "--char"),
            text_charset: get_charset(p.get_str("--text-charset"))?
                .as_bytes()
                .to_vec(),
            known_plain: p.get_str("--known-plaintext").bytes().collect(),
            threshold: parse_optional_int(&p, "--threshold"),
        }),
        Err(e) => Err(XorError::ArgError { msg: e.to_string() }),
    }
    // .unwrap_or_else(|e| e.exit());
}
