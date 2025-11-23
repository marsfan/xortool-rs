/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
mod args;
mod charset;
mod colors;
mod error;
pub mod libcolors;
mod routine;
pub mod tool_main;
pub mod tool_xor;

// https://stackoverflow.com/a/27841363
const VERSION: &str = env!("CARGO_PKG_VERSION");
