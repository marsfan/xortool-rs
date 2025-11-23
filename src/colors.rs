/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use std::sync::LazyLock;

use crate::libcolors::color;
pub static C_RESET: LazyLock<String> = LazyLock::new(|| color("", "", ""));
pub static C_FATAL: LazyLock<String> = LazyLock::new(|| color("red", "", ""));
pub static C_WARN: LazyLock<String> = LazyLock::new(|| color("yellow", "", ""));
pub static C_KEYLEN: LazyLock<String> = LazyLock::new(|| color("green", "", ""));
pub static C_PROB: LazyLock<String> = LazyLock::new(|| color("white", "", ""));
pub static C_BEST_KEYLEN: LazyLock<String> = LazyLock::new(|| color("green", "", "bold"));
pub static C_BEST_PROB: LazyLock<String> = LazyLock::new(|| color("white", "", "bold"));
pub static C_DIV: LazyLock<String> = LazyLock::new(|| color("", "", "bold"));
pub static C_KEY: LazyLock<String> = LazyLock::new(|| color("red", "", "bold"));
pub static C_COUNT: LazyLock<String> = LazyLock::new(|| color("yellow", "", "bold"));
