/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Command line argument parsing utilities.
use clap::Parser;

use crate::{charset::get_charset, error::XorError};

/// Parse `most_frequent_char` argument into a byte
///
/// # Arguments
///   * `arg`: The argument to parse
///
/// # Returns
///   * Argument converted to a byte
///
/// # Errors
///   Returns an error of the supplied argument text is empty
///   or if the supplied argument is not a single character, or a hex
///   value (i.e. prefixed with `0x` or `\\x`)
fn parse_most_frequent(mut arg: &str) -> Result<u8, XorError> {
    if arg.len() == 1 {
        return Ok(arg.bytes().collect::<Vec<u8>>()[0]);
    }
    if arg.starts_with("0x") {
        arg = arg.strip_prefix("0x").unwrap();
    }
    if arg.starts_with("\\x") {
        arg = arg.strip_prefix("\\x").unwrap();
    }
    if arg.is_empty() {
        return Err(XorError::ArgParser {
            msg: "Empty Char".to_owned(),
        });
    }
    if arg.len() > 2 {
        return Err(XorError::ArgParser {
            msg: "Char can only be a char letter or hex".to_owned(),
        });
    }
    Ok(u8::from_str_radix(arg, 16).unwrap())
}

/// Convert a string to a vector of bytes
///
/// # Arguments
///   * `arg`: The value to convert
///
/// # Returns
///   The input value, as a vector of bytes
///
/// # Errors
///   Returns `XorError::ArgParser` if an empty string is supplied.
fn str_to_bytes(arg: &str) -> Result<Vec<u8>, XorError> {
    if arg.is_empty() {
        Err(XorError::ArgParser {
            msg: "Empty String".to_owned(),
        })
    } else {
        Ok(arg.as_bytes().to_vec())
    }
}

/// Structure holding the parsed command line arguments
#[expect(
    clippy::struct_excessive_bools,
    reason = "This structure holds CLI args, lots of bools are expected as they are for flags."
)]
#[derive(Parser, Default)]
#[command(version, about, long_about = None, about="A tool to do some xor analysis:\n- Guess the key length (based on count of equal chars)\n- Guess the key (based on knowledge of most frequent char)", after_help="

Notes:
Text character set:
    * Pre-defined sets: printable, base32, base64
    * Custom sets:
    - a: lowercase chars
    - A: uppercase chars
    - 1: digits
    - !: special chars
    - *: printable chars

Examples:
xortool file.bin
xortool -l 11 -c 20 file.bin
xortool -x -c ' ' file.hex
xortool -b -f -l 23 -t base64 message.enc
xortool -r 80 -p \"flag{\" -c ' ' message.enc
")]
pub struct Parameters {
    /// Whether or not to brute force all possible most frequent characters
    #[arg(short, long, help = "Brute force all possible most frequent chars")]
    pub brute_chars: bool,

    /// Whether or not to brute force all possible most frequent printable characters
    #[arg(
        short = 'o',
        long,
        help = "Same as -b but will only check printable chars"
    )]
    pub brute_printable: bool,

    /// Name of the file to read in from
    #[arg(default_value = "-")]
    pub filename: String,

    /// Whether or not to filter outputs based on the charset.
    #[arg(short, long, help = "filter outputs based on the charset")]
    pub filter_output: bool,

    /// Whether or not the input is a hex-encoded string.
    #[arg(short = 'x', long = "hex", help = "input is hex-encoded str")]
    pub input_is_hex: bool,

    /// Optional known length of the key
    #[arg(
        short = 'l',
        long = "key-length",
        value_name = "LEN",
        help = "Length of the key"
    )]
    pub known_key_length: Option<i32>,

    /// Maximum key length to probe.
    #[arg(
        short = 'm',
        long = "max-keylen",
        value_name = "MAXLEN",
        help = "Maximum key length to probe [default: 65]",
        default_value = "65"
    )]
    pub max_key_length: Option<i32>,

    /// Known most frequent character in the plaintext
    #[arg(
        short = 'c',
        long = "char",
        value_name = "CHAR",
        value_parser = parse_most_frequent,
        help = "Most frequent char (one char or hex code)"
    )]
    pub most_frequent_char: Option<u8>,

    /// Target text character set
    #[arg(
        short = 't',
        long = "text-charset",
        value_name = "CHARSET",
        help = "Target text character set [default: printable]",
        value_parser = get_charset,
        default_value="printable"
    )]
    #[expect(
        clippy::absolute_paths,
        reason = "This needs to be fully qualified to work properly. See https://github.com/clap-rs/clap/issues/4481#issuecomment-1314475143"
    )]
    pub text_charset: std::vec::Vec<u8>,

    /// Known plaintext to use for decoding
    #[arg(
        short = 'p',
        long = "known-plaintext",
        value_name = "PLAIN",
        value_parser = str_to_bytes,
        help = "Use known plaintext for decoding"
    )]
    #[expect(
        clippy::absolute_paths,
        reason = "This needs to be fully qualified to work properly. See https://github.com/clap-rs/clap/issues/4481#issuecomment-1314475143"
    )]
    pub known_plain: Option<std::vec::Vec<u8>>,

    /// Threshold validity percentage (default: 95)
    #[arg(
        short = 'r',
        long = "threshold",
        value_name = "PERCENT",
        help = "Threshold validity percentage [default: 95]"
    )]
    pub threshold: Option<i32>,
}
