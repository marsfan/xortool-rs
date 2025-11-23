/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use lazy_static::lazy_static;
use std::{collections::HashMap, env, fmt::Write as _, string};
// FIXME: Probably could replace this whole thing with some sort of crate.

lazy_static! {
    static ref BASH_ATTRIBUTES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("regular", "0");
        m.insert("bold", "1");
        m.insert("underline", "4");
        m.insert("strike", "9");
        m.insert("light", "1");
        m.insert("dark", "2");
        m.insert("invert", "7");// invert bg and fg
        m
    };
    static ref BASH_COLORS: HashMap<&'static str, &'static str> = {
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
    };

    static ref BASH_BGCOLORS: HashMap<&'static str, &'static str> = {
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
    };
}

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

pub fn is_bash() -> bool {
    match env::var("SHELL") {
        Ok(v) => v.ends_with("bash"),
        Err(_) => false,
    }
}

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
