/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use std::{env, error::Error, fmt::Display};

#[derive(Debug)]
pub enum XorError {
    AnalysisError { msg: String },
    ArgError { msg: String },
    CharsetError { charset: char },
    // FIXME: Nothing raises this right now. Need to link it to file input ops
    IOError { msg: String },
    MkdirError { msg: String },
    // FIXME: Nothing raises this right now. Need to link to relevant stuff
    UnicodeDecodeError { msg: String },
}

impl Display for XorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
