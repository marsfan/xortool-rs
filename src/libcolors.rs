use lazy_static::lazy_static;
use std::collections::HashMap;
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
    println!("");

    println!(
        "{header}       Colors and backgrounds:      {}",
        color("", "", "")
    );
    for c in _keys_sorted_by_value(BASH_COLORS.clone()) {
        let c1 = color(&c, "", "");
        let c2_name = if c != "white" { "white" } else { "black" };
        let c2 = color(c2_name, &c, "");
        println!(
            "{c:<10}{c1}colored text{}    {c2}background{}",
            color("", "", ""),
            color("", "", "")
        );
    }
    println!("");

    println!(
        "{header}            Attributes:             {}",
        color("", "", "")
    );
    for c in _keys_sorted_by_value(BASH_ATTRIBUTES.clone()) {
        let c1 = color("red", "", &c);
        let c2 = color("white", "", &c);
        println!(
            "{c:<10}{c1}red text{}     {c2}white text{}",
            color("", "", ""),
            color("", "", "")
        );
    }
    println!("");
}

pub fn color(color: &str, bgcolor: &str, attrs: &str) -> String {
    if !is_bash() {
        return String::from("");
    }

    let mut ret = String::from("\x1b[0");
    if attrs != "" {
        for attr in attrs.to_lowercase().split_whitespace() {
            // FIXME: Something similarr tto pythons strip method instead?
            let attr = attr.replace(",", "").replace("+", "").replace("|", "");
            if !BASH_ATTRIBUTES.contains_key(attr.as_str()) {
                panic!("Unknown color attribute: {attr}");
            }
            ret.push_str(";");
            ret.push_str(BASH_ATTRIBUTES.get(attr.as_str()).unwrap());
        }
    }

    if color != "" {
        if !BASH_COLORS.contains_key(&color) {
            panic!("Unknown color: {color}");
        }
        ret.push_str(";");
        ret.push_str(BASH_COLORS.get(&color).unwrap());
    }

    if bgcolor != "" {
        if !BASH_BGCOLORS.contains_key(&bgcolor) {
            panic!("Unknown background color: {bgcolor}");
        }
        ret.push_str(";");
        ret.push_str(BASH_BGCOLORS.get(&bgcolor).unwrap());
    }

    ret.push_str("m");
    ret
}

pub fn is_bash() -> bool {
    // FIXME: Actually implement this properly.
    return false;
}

fn _keys_sorted_by_value(adict: HashMap<&'static str, &'static str>) -> Vec<String> {
    let mut keys: Vec<String> = adict.keys().map(|v| v.to_string()).collect();
    keys.sort_by_key(|v| adict.get(v.as_str()));
    keys
}
