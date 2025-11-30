/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Custom error type for the tool
use std::{env, error::Error, fmt};

/// Enumeration of errors the tool may experience.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum XorError {
    /// An error occurred during analysis of the data
    Analysis {
        /// Message with further details about the error.
        msg: String,
    },
    /// Incorrect short form for a charset
    Charset {
        /// The invalid short form supplied.
        charset: char,
    },
    /// Error occurred with data input/output
    IO {
        /// Message with further details about the error
        msg: String,
    },
    /// An error occurred when trying to create a directory
    Mkdir {
        /// Message with further details about the error
        msg: String,
    },
    /// An error occurred decoding a string to bytes
    UnicodeDecode {
        /// Message with further details about the error
        msg: String,
    },

    /// An error occurred when parsing command line arguments.
    ArgParser {
        /// Message with further details about the errorr
        msg: String,
    },
}

impl fmt::Display for XorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (type_str, details) = match self {
            Self::Analysis { msg } => ("Analysis error", msg.clone()),
            Self::Charset { charset } => (
                "Bad charset",
                format!(" ('Bad character set: ', '{charset}') "),
            ),
            Self::IO { msg } => ("Can't load file", msg.clone()),
            Self::Mkdir { msg } => ("Can't create directory", msg.clone()),
            Self::UnicodeDecode { msg } => ("Input is not hex", msg.clone()),
            Self::ArgParser { msg } => ("Bad argument", msg.clone()),
        };
        if env::consts::OS == "windows" {
            write!(f, "[ERROR] {type_str}:\r\n\t{details}")
        } else {
            write!(f, "[ERROR] {type_str}:\n\t{details}")
        }
    }
}
impl Error for XorError {}

#[expect(
    clippy::absolute_paths,
    reason = "Since we already use a different error, we use absolute path here to disambiguate them."
)]
impl From<clap::error::Error> for XorError {
    fn from(value: clap::error::Error) -> Self {
        if let Some(v) = value.source() {
            if let Some(downcast) = v.downcast_ref::<XorError>() {
                downcast.clone()
            } else {
                XorError::ArgParser {
                    msg: value.render().to_string(),
                }
            }
        } else {
            XorError::ArgParser {
                msg: value.render().to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_analysis_error() {
        let err = XorError::Analysis {
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
        let err = XorError::ArgParser {
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
        let err = XorError::Charset { charset: 'Q' };

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
        let err = XorError::IO {
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
        let err = XorError::Mkdir {
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
        let err = XorError::UnicodeDecode {
            msg: String::from("ABCD"),
        };

        if env::consts::OS == "windows" {
            assert_eq!(err.to_string(), "[ERROR] Input is not hex:\r\n\tABCD");
        } else {
            assert_eq!(err.to_string(), "[ERROR] Input is not hex:\n\tABCD");
        }
    }
}
