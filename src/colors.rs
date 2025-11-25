/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Custom bash color definitions for the tool
use std::sync::LazyLock;

use crate::libcolors::color;
/// Reset the console color back to normal
pub static C_RESET: LazyLock<String> = LazyLock::new(|| color("", "", ""));

/// Console color used to display fatal error messages
pub static C_FATAL: LazyLock<String> = LazyLock::new(|| color("red", "", ""));

/// Console color to display warnings
pub static C_WARN: LazyLock<String> = LazyLock::new(|| color("yellow", "", ""));

/// Console color to display key lengths in
pub static C_KEYLEN: LazyLock<String> = LazyLock::new(|| color("green", "", ""));

/// Console color to display probability in
pub static C_PROB: LazyLock<String> = LazyLock::new(|| color("white", "", ""));

/// Console color to display best key length in
pub static C_BEST_KEYLEN: LazyLock<String> = LazyLock::new(|| color("green", "", "bold"));

/// Console color to display best probability in
pub static C_BEST_PROB: LazyLock<String> = LazyLock::new(|| color("white", "", "bold"));

/// Console color to display key-length divisor in
pub static C_DIV: LazyLock<String> = LazyLock::new(|| color("", "", "bold"));

/// Console color to display keys in
pub static C_KEY: LazyLock<String> = LazyLock::new(|| color("red", "", "bold"));

/// Console color to display counts in.
pub static C_COUNT: LazyLock<String> = LazyLock::new(|| color("yellow", "", "bold"));
