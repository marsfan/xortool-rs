/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Support for bash colors
// FIXME: Probably could replace this whole thing with some sort of crate.
use std::{collections::HashMap, env, fmt::Write as _, string, sync::LazyLock};

/// Table of different attributes supported by bash, and their integer codes
static BASH_ATTRIBUTES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("regular", "0");
    m.insert("bold", "1");
    m.insert("underline", "4");
    m.insert("strike", "9");
    m.insert("light", "1");
    m.insert("dark", "2");
    m.insert("invert", "7"); // invert bg and fg
    m
});

/// Table of text colors supported by bash, and their integer codes.
static BASH_COLORS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("black", "30");
    m.insert("red", "31");
    m.insert("green", "32");
    m.insert("yellow", "33");
    m.insert("blue", "34");
    m.insert("purple", "35");
    m.insert("cyan", "36");
    m.insert("white", "37");
    m
});

/// Table of background colors supported by bash, and their integer codes
static BASH_BGCOLORS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("black", "40");
    m.insert("red", "41");
    m.insert("green", "42");
    m.insert("yellow", "43");
    m.insert("blue", "44");
    m.insert("purple", "45");
    m.insert("cyan", "46");
    m.insert("white", "47");
    m
});

/// Main function for the colortest program.
pub fn _main() {
    let header = color("white", "black", "dark");
    println!();

    println!(
        "{header}       Colors and backgrounds:      {}",
        color("", "", "")
    );
    for c in keys_sorted_by_value(&BASH_COLORS) {
        let c1 = color(&c, "", "");
        let c2_name = if c == "white" { "black" } else { "white" };
        let c2 = color(c2_name, &c, "");
        println!(
            "{c:<10}{c1}colored text{}    {c2}background{}",
            color("", "", ""),
            color("", "", "")
        );
    }
    println!();

    println!(
        "{header}            Attributes:             {}",
        color("", "", "")
    );
    for c in keys_sorted_by_value(&BASH_ATTRIBUTES) {
        let c1 = color("red", "", &c);
        let c2 = color("white", "", &c);
        println!(
            "{c:<10}{c1}red text{}     {c2}white text{}",
            color("", "", ""),
            color("", "", "")
        );
    }
    println!();
}

/// Create a single POSIX color/attribute setting string
///
/// # Arguments
///  * `color`: The foreground color to set
///  * `bgcolor`: The background color to set
///  * `attr`: Attributes to set
///
/// # Returns
///   Text for setting a POSIX terminal's colors and attributes.
///
/// # Panics
///   This function will panic if an unknown color, background color, or
///   attribute is supplied
pub fn color(color: &str, bgcolor: &str, attrs: &str) -> String {
    if !is_bash() {
        return String::new();
    }

    let mut ret = String::from("\x1b[0");
    if !attrs.is_empty() {
        for attr in attrs.to_lowercase().split_whitespace() {
            // FIXME: Something similar tto pythons strip method instead?
            let attr = attr.replace([',', '+', '|'], "");
            assert!(
                BASH_ATTRIBUTES.contains_key(attr.as_str()),
                "Unknown color attribute: {attr}"
            );
            write!(ret, ";{}", BASH_ATTRIBUTES.get(attr.as_str()).unwrap()).unwrap();
        }
    }

    if !color.is_empty() {
        assert!(BASH_COLORS.contains_key(&color), "Unknown color: {color}");

        write!(ret, ";{}", BASH_COLORS.get(&color).unwrap()).unwrap();
    }

    if !bgcolor.is_empty() {
        assert!(
            BASH_BGCOLORS.contains_key(&bgcolor),
            "Unknown background color: {bgcolor}"
        );
        write!(ret, ";{}", BASH_BGCOLORS.get(&bgcolor).unwrap()).unwrap();
    }

    ret.push('m');
    ret
}

/// Check if running in a bash shell
///
/// # Returns
///   Boolean indicating if running in a bash shell
pub fn is_bash() -> bool {
    match env::var("SHELL") {
        Ok(v) => v.ends_with("bash"),
        Err(_) => false,
    }
}

/// Get the keys in a hashmap, sorted by their values.
///
/// # Arguments:
///   * `adict`: The hashmap to get the keys from
///
/// # Returns
///   Keys from the input hashmap, sorted by the values.
fn keys_sorted_by_value(adict: &HashMap<&'static str, &'static str>) -> Vec<String> {
    let mut keys: Vec<String> = adict.keys().map(string::ToString::to_string).collect();
    keys.sort_by_key(|v| adict.get(v.as_str()));
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_sorted_by_value() {
        let mut data = HashMap::new();
        data.insert("key1", "C");
        data.insert("key2", "A");
        data.insert("key3", "B");

        assert_eq!(keys_sorted_by_value(&data), vec!["key2", "key3", "key1"]);
    }
}
