/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Core logic for xortool-xor
use std::{
    env, fs, io,
    io::{Read as _, Write as _, stdout},
    process::exit,
    vec::Vec,
};

use clap::{ArgAction, CommandFactory as _, Parser};
use unicode_escape::decode;

use crate::error::XorError;

/// Structure holding the parsed command line arguments
#[derive(Parser, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "This structure holds CLI args, lots of bools are expected as they are for flags."
)]
#[command(
    version,
    about,
    about = "xor strings",
    after_help = "example: xor -s lol -h 414243 -f /etc/passwd",
    disable_help_flag = true
)]
pub struct Parameters {
    /// String with \\xAF escapes
    #[arg(short='s', value_parser=from_str)]
    pub string: Vec<Vec<u8>>,

    /// Raw strings
    #[arg( short='r', value_parser=from_raw_str)]
    pub raw_string: Vec<Vec<u8>>,

    /// Hex-encoded string (non-letterdigit chars are stripped)
    #[arg(short='h', value_parser=from_hex_str)]
    pub hex_string: Vec<Vec<u8>>,

    /// Read dta from file (- for stdin)
    #[arg(short='f', value_parser=from_file)]
    pub file: Vec<Vec<u8>>,

    /// Newline at the end (default)
    #[arg(long="newline", action=ArgAction::SetTrue, overrides_with="no_newline")]
    pub newline: bool,

    /// No newline at the end
    #[arg(short = 'n', long = "no-newline", action=ArgAction::SetFalse, overrides_with="newline")]
    pub no_newline: bool,

    /// Do not pad (default)
    #[arg(long, action=ArgAction::SetTrue, overrides_with="no_cycle")]
    pub cycle: bool,

    /// Pad smaller strings with null bytes
    #[arg(long="no-cycle", visible_alias ="nc", action=ArgAction::SetFalse, overrides_with="cycle")]
    pub no_cycle: bool,

    /// Print help
    #[clap(long, action = clap::ArgAction::HelpLong)]
    pub help: Option<bool>,
}

/// Main function for xortool-xor
///
/// # Arguments
///   * `args`: Optional vector of the command line arguments to parse
///     If not provided, arguments are read from stdin instead.
///
/// # Panics
///   Will panic if an error occurs when parsing the command line arguments.
pub fn main(args: Option<Vec<String>>) {
    let param = match args {
        Some(a) => Parameters::parse_from(a),
        None => Parameters::parse(),
    };

    let cycle = param.cycle || param.no_cycle;
    let newline = param.newline || param.no_newline;

    let mut datas = Vec::new();
    datas.extend_from_slice(&param.string);
    datas.extend_from_slice(&param.raw_string);
    datas.extend_from_slice(&param.hex_string);
    datas.extend_from_slice(&param.file);

    if datas.is_empty() {
        let line_end = if env::consts::OS == "windows" {
            "\r\n"
        } else {
            "\n"
        };
        eprint!("error: no data given{line_end}{line_end}");
        eprint!("{}{line_end}", Parameters::command().render_help());
        exit(1)
    }

    let result = xor(datas, cycle);
    // FIXME: Replace these unwraps with conversion to XorError::IO
    stdout().write_all(&result).unwrap();
    if newline {
        stdout().write_all("\n".as_bytes()).unwrap();
    }
    stdout().flush().unwrap();
}

/// Compute xor-encoded value of all of the data
///
/// # Arguments
///   * `args`: 2D Vector of all of the data to xor encode
///   * `cycle`: Whether to use the longest of the data components for
///     iteration length (true), or the length of each individual component (false)
///
/// # Returns
///  xor-encoding of all of the data
fn xor(mut args: Vec<Vec<u8>>, cycle: bool) -> Vec<u8> {
    args.sort_by_key(Vec::len);
    // Pop First then reverse is the same as popping first item after reversing
    let mut res = args.pop().unwrap();
    args.reverse();
    let maxlen = res.len();
    for s in args {
        let slen = s.len();
        let range_end = if cycle { maxlen } else { slen };
        for i in 0..range_end {
            res[i] ^= s[i % slen];
        }
    }
    res
}

/// Convert a string into a vector of bytes, decoding escape sequences
///
/// # Arguments
///   * `s`: The string to convert
///
/// # Returns
///   Vector of the bytes of the string.
///
/// # Errors
///   Returns `XorError::ArgParser` if the supplied string is empty
fn from_str(s: &str) -> Result<Vec<u8>, XorError> {
    if s.is_empty() {
        Err(XorError::ArgParser {
            msg: "Empty String".to_owned(),
        })
    } else {
        match decode(s) {
            Ok(v) => Ok(v.bytes().collect()),
            Err(e) => Err(XorError::UnicodeDecode { msg: e.to_string() }),
        }
    }
}

/// Parse a raw string to bytes
///
/// # Arguments
///   * `arg`: The string to parse
///
/// # Returns
///   Vector of bytes from the parsed string
///
/// # Errors
///   Returns `XorError::ArgParser` if the supplied string is empty
fn from_raw_str(arg: &str) -> Result<Vec<u8>, XorError> {
    if arg.is_empty() {
        Err(XorError::ArgParser {
            msg: "Empty String".to_owned(),
        })
    } else {
        Ok(arg.as_bytes().to_vec())
    }
}

/// Parse from a string of hex characters to bytes
///
/// # Arguments
///   * `arg`: The string to parse
///
/// # Returns
///   Hex characters converted to bytes
///
/// # Errors
///   Returns `XorError::ArgParser` if the supplied string is empty
fn from_hex_str(arg: &str) -> Result<Vec<u8>, XorError> {
    if arg.is_empty() {
        Err(XorError::ArgParser {
            msg: "Empty String".to_owned(),
        })
    } else {
        Ok(arg
            .replace(' ', "")
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|c| u8::from_str_radix(&c.iter().collect::<String>(), 16).unwrap())
            .collect())
    }
}

/// Read from a file into a vector of bytes
///
/// # Arguments
///   * `file`: The file to read from. If "-", will read from stdin instead
///
/// # Returns
///   Vector of the bytes that were read in.
///
/// # Errors
///   Returns `XorError::ArgParser` if the supplied string is empty
fn from_file(s: &str) -> Result<Vec<u8>, XorError> {
    if s.is_empty() {
        Err(XorError::ArgParser {
            msg: "Empty String".to_owned(),
        })
    } else {
        if s == "-" {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            return Ok(buf);
        }
        Ok(fs::read(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            from_str("Hello \\tWorld!"),
            Ok("Hello \tWorld!".as_bytes().to_vec())
        )
    }

    #[test]
    fn test_from_raw_string() {
        assert_eq!(
            from_raw_str("Hello \\tWorld!"),
            Ok("Hello \\tWorld!".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_from_hex_str() {
        assert_eq!(
            from_hex_str("48 65 6C 6C 6F 20 57 6F 72 6C 64"),
            Ok("Hello World".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_from_file() {
        assert_eq!(
            from_file("tests/small_file.txt"),
            Ok("Hello World!".as_bytes().to_vec())
        );
    }
}
