/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use std::collections::HashMap;

use crate::libcolors::color;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref C_RESET: String = color("", "", "");
    pub static ref C_FATAL: String = color("red", "", "");
    pub static ref C_WARN: String = color("yellow", "", "");
    pub static ref C_KEYLEN: String = color("green", "", "");
    pub static ref C_PROB: String = color("white", "", "");
    pub static ref C_BEST_KEYLEN: String = color("green", "", "bold");
    pub static ref C_BEST_PROB: String = color("white", "", "bold");
    pub static ref C_DIV: String = color("", "", "bold");
    pub static ref C_KEY: String = color("red", "", "bold");
    pub static ref C_BOLD: String = color("", "", "bold");
    pub static ref C_COUNT: String = color("yellow", "", "bold");
    pub static ref COLORS: HashMap<&'static str, String> = {
        let mut m: HashMap<&'static str, String> = HashMap::new();
        m.insert("C_RESET", C_RESET.to_string());
        m.insert("C_FATAL", C_FATAL.to_string());
        m.insert("C_WARN", C_WARN.to_string());
        m.insert("C_KEYLEN", C_KEYLEN.to_string());
        m.insert("C_PROB", C_PROB.to_string());
        m.insert("C_BEST_KEYLEN", C_BEST_KEYLEN.to_string());
        m.insert("C_BEST_PROB", C_BEST_PROB.to_string());
        m.insert("C_DIV", C_DIV.to_string());
        m.insert("C_KEY", C_KEY.to_string());
        m.insert("C_BOLD", C_BOLD.to_string());
        m.insert("C_COUNT", C_COUNT.to_string());
        m
    };
}
