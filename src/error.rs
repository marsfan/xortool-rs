/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Custom error type for the tool
use std::{env, error::Error, fmt};

/// Enumeration of errors the tool may experience.
#[derive(Debug, PartialEq, Eq)]
pub enum XorError {
    /// An error occurred during analysis of the data
    AnalysisError {
        /// Message with further details about the error.
        msg: String,
    },
    /// An error occurred while parsing arguments
    ArgError {
        /// Message with further details about the error
        msg: String,
    },
    /// Incorrect short form for a charset
    CharsetError {
        /// The invalid short form supplied.
        charset: char,
    },
    /// Error occurred with data input/output
    IOError {
        /// Message with further details about the error
        msg: String,
    },
    /// An error occurred when trying to create a directory
    MkdirError {
        /// Message with further details about the error
        msg: String,
    },
    /// An error occurred decoding a string to bytes
    UnicodeDecodeError {
        /// Message with further details about the error
        msg: String,
    },
}

impl fmt::Display for XorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (type_str, details) = match self {
            Self::AnalysisError { msg } => ("Analysis error", msg.clone()),
            Self::ArgError { msg } => ("Bad argument", msg.clone()),
            Self::CharsetError { charset } => (
                "Bad charset",
                format!(" ('Bad character set: ', '{charset}') "),
            ),
            Self::IOError { msg } => ("Can't load file", msg.clone()),
            Self::MkdirError { msg } => ("Can't create directory", msg.clone()),
            Self::UnicodeDecodeError { msg } => ("Input is not hex", msg.clone()),
        };
        if env::consts::OS == "windows" {
            write!(f, "[ERROR] {type_str}:\r\n\t{details}")
        } else {
            write!(f, "[ERROR] {type_str}:\n\t{details}")
        }
    }
}
impl Error for XorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_analysis_error() {
        let err = XorError::AnalysisError {
            msg: String::from("ABCD"),
        };

        if env::consts::OS == "windows" {
            assert_eq!(err.to_string(), "[ERROR] Analysis error:\r\n\tABCD");
        } else {
            assert_eq!(err.to_string(), "[ERROR] Analysis error:\n\tABCD");
        }
    }

    #[test]
    fn test_fmt_arg_error() {
        let err = XorError::ArgError {
            msg: String::from("ABCD"),
        };

        if env::consts::OS == "windows" {
            assert_eq!(err.to_string(), "[ERROR] Bad argument:\r\n\tABCD");
        } else {
            assert_eq!(err.to_string(), "[ERROR] Bad argument:\n\tABCD");
        }
    }

    #[test]
    fn test_fmt_charset_error() {
        let err = XorError::CharsetError { charset: 'Q' };

        if env::consts::OS == "windows" {
            assert_eq!(
                err.to_string(),
                "[ERROR] Bad charset:\r\n\t ('Bad character set: ', 'Q') "
            );
        } else {
            assert_eq!(
                err.to_string(),
                "[ERROR] Bad charset:\n\t ('Bad character set: ', 'Q') "
            );
        }
    }

    #[test]
    fn test_ioerror() {
        let err = XorError::IOError {
            msg: String::from("ABCD"),
        };

        if env::consts::OS == "windows" {
            assert_eq!(err.to_string(), "[ERROR] Can't load file:\r\n\tABCD");
        } else {
            assert_eq!(err.to_string(), "[ERROR] Can't load file:\n\tABCD");
        }
    }

    #[test]
    fn test_mkdir_error() {
        let err = XorError::MkdirError {
            msg: String::from("ABCD"),
        };

        if env::consts::OS == "windows" {
            assert_eq!(err.to_string(), "[ERROR] Can't create directory:\r\n\tABCD");
        } else {
            assert_eq!(err.to_string(), "[ERROR] Can't create directory:\n\tABCD");
        }
    }

    #[test]
    fn test_unicode_decode_error() {
        let err = XorError::UnicodeDecodeError {
            msg: String::from("ABCD"),
        };

        if env::consts::OS == "windows" {
            assert_eq!(err.to_string(), "[ERROR] Input is not hex:\r\n\tABCD");
        } else {
            assert_eq!(err.to_string(), "[ERROR] Input is not hex:\n\tABCD");
        }
    }
}
