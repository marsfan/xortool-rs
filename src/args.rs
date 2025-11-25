/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Command line argument parsing utilities.
use docopt::{ArgvMap, Docopt};
use std::env;

use crate::{charset::get_charset, error::XorError};

/// Structure holding parsed command line options
#[derive(Default)]
pub struct Parameters {
    /// Whether or not to brute force all possible most frequent characters
    pub brute_chars: bool,

    /// Whether or not to brute force all possible most frequent printable characters
    pub brute_printable: bool,

    /// Name of the file to read in from
    pub filename: String,

    /// Whether or not to filter outputs based on the charsett.
    pub filter_output: bool,

    /// Whether or not the input is a hex-encoded string.
    pub input_is_hex: bool,

    /// Optional known length of the key
    pub known_key_length: Option<i32>,

    /// Maximum key length to probe.
    pub max_key_length: Option<i32>,

    /// Known most frequent character in the plaintext
    pub most_frequent_char: Option<u8>,

    /// Target text character set
    pub text_charset: Vec<u8>,

    /// Known plaintext to use for decoding
    pub known_plain: Vec<u8>,

    /// Threshold validity percentage (default: 95)
    pub threshold: Option<i32>,
}

/// Parse an argument into a u8 character.
///
/// # Arguments
///   * `parsed`: The parsed arguments
///   * `arg`: The argument to parse
///
/// # Returns
///   The character converted to a `u8`, or `None` if the given argument
///   was not provided.
fn parse_char(parsed: &ArgvMap, arg: &str) -> Option<u8> {
    let mut ch = parsed.get_str(arg);
    if ch.is_empty() {
        None
    } else {
        if ch.len() == 1 {
            return Some(ch.bytes().collect::<Vec<u8>>()[0]);
        }
        if ch.starts_with("0x") {
            ch = ch.strip_prefix("0x").unwrap();
        }
        if ch.starts_with("\\x") {
            ch = ch.strip_prefix("\\x").unwrap();
        }
        assert!(!ch.is_empty(), "Empty Char");
        assert!(ch.len() <= 2, "Char can be only a char letter or hex");
        Some(u8::from_str_radix(ch, 16).unwrap())
    }
}

/// Parse an optional argument to an integer.
///
/// # Arguments
///   * `parsed`: The parsed arguments
///   * `arg`: The argument to parse
///
/// # Returns
///   The argument as an integer, or `None` if the given argument
///   was not provided.
fn parse_optional_int(parsed: &ArgvMap, arg: &str) -> Option<i32> {
    let value = parsed.get_str(arg);
    if value.is_empty() {
        None
    } else {
        Some(value.parse().unwrap())
    }
}

/// Parse parameters from the commandline using docopt
///
/// # Arguments
///   * `doc`: The documentation to pass to docopt to use for parsing the
///     arguments
///   * `version`: The version number of the tool
///   * `args`: The arguments to parse, or None to parse from the command
///     line instead.
///
/// # Returns
///   The parsed command line options, or a `XorError` instance on error.
pub fn parse_parameters(
    doc: &str,
    version: &str,
    args: Option<Vec<String>>,
) -> Result<Parameters, XorError> {
    let args = match args {
        Some(a) => a,
        None => env::args().collect(),
    };
    let p =
        Docopt::new(doc).and_then(|dopt| dopt.version(Some(version.to_owned())).argv(args).parse());
    match p {
        Ok(p) => Ok(Parameters {
            brute_chars: p.get_bool("--brute-chars"),
            brute_printable: p.get_bool("--brute-printable"),

            filename: if p.get_str("FILE").is_empty() {
                "-".to_owned()
            } else {
                p.get_str("FILE").to_owned()
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
        Err(e) => Err(XorError::Arg { msg: e.to_string() }),
    }
}
