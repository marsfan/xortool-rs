/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use std::{
    env,
    io::{Read, Write, stdout},
    process::exit,
};
use unicode_escape::decode;

use getopt::Opt;
use lazy_static::lazy_static;

use crate::VERSION;

lazy_static! {
    static ref DOC: String = format!(
        "
xortool-xor {VERSION}
xor strings
options:
    -s  -  string with \\xAF escapes
    -r  -  raw string
    -h  -  hex-encoded string (non-letterdigit chars are stripped)
    -f  -  read data from file (- for stdin)

    --newline -  newline at the end (default)
    -n / --no-newline -  no newline at the end
    --cycle - do not pad (default)
    --no-cycle / --nc  -  pad smaller strings with null bytes
example: xor -s lol -h 414243 -f /etc/passwd
"
    );
}

pub fn main(args: Option<Vec<String>>) {
    let mut cycle = true;
    let mut newline = true;

    // Use input arg if provided, otherwise read from stdin
    let stdin_args = match args {
        Some(a) => a,
        None => std::env::args().collect(),
    };

    let no_doubles: Vec<String> = stdin_args
        .clone()
        .into_iter()
        .filter(|v| !v.starts_with("--"))
        .collect();
    let mut opts = getopt::Parser::new(&no_doubles, "ns:r:h:f:");
    let mut datas = Vec::new();
    let mut collected_args: Vec<(String, String)> = Vec::new();

    // This process is a bit different from tool_xor.py, due to not really
    // having getopt in rust that can handle long form args.
    // So we use a getopt crate, and then gather all the other args that
    // start with a double dash, then we parse through like in the python code.
    // It also means we don't currently error out for unknown arrg types
    loop {
        match opts.next().transpose().unwrap() {
            None => break,
            Some(opt) => match opt {
                Opt(key, Some(arg)) => collected_args.push((key.to_string(), arg)),
                Opt(key, None) => collected_args.push((key.to_string(), String::new())),
            },
        }
    }
    // Collect long args. Luckily there's no long args that take parameters, so
    // this is really easy.
    for arg in stdin_args.iter() {
        if arg.starts_with("--") {
            collected_args.push((arg.clone(), String::new()));
        }
    }

    // Now we are actually back on course.
    for (c, val) in collected_args {
        if c == "--cycle" {
            cycle = true;
        } else if ["--no-cycle", "--nc"].contains(&c.as_str()) {
            cycle = false;
        } else if c == "--newline" {
            newline = true;
        } else if ["n", "--no-newline"].contains(&c.as_str()) {
            newline = false;
        } else {
            datas.push(arg_data(&c, &val));
        }
    }

    if datas.is_empty() {
        let line_end = if env::consts::OS == "windows" {
            "\r\n"
        } else {
            "\n"
        };
        let msg = if env::consts::OS == "windows" {
            (*DOC).replace("\n", "\r\n")
        } else {
            DOC.to_string()
        };
        eprint!("error: no data given{line_end}");
        eprint!("{msg}{line_end}");
        exit(1)
    }

    let result = xor(datas, cycle);
    stdout().write_all(&result).unwrap();
    if newline {
        stdout().write_all("\n".as_bytes()).unwrap();
    }
    stdout().flush().unwrap();
}

fn xor(mut args: Vec<Vec<u8>>, cycle: bool) -> Vec<u8> {
    args.sort_by_key(|v| v.len());
    // Pop First then reverse is the same as popping first item after reversing
    let mut res = args.pop().unwrap();
    args.reverse();
    let maxlen = res.len();
    for s in args {
        let slen = s.len();
        let range_end = if cycle { maxlen } else { slen };
        for i in 0..range_end {
            res[i] ^= s[i % slen];
        }
    }
    res
}

fn from_str(s: &str) -> Vec<u8> {
    decode(s).unwrap().bytes().collect()
}

fn from_file(s: &str) -> Vec<u8> {
    if s == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).unwrap();
        return buf;
    }
    std::fs::read(s).unwrap()
}

fn arg_data(opt: &str, s: &str) -> Vec<u8> {
    let line_end = if env::consts::OS == "windows" {
        "\r\n"
    } else {
        "\n"
    };
    match opt {
        "s" => from_str(s),
        "r" => s.bytes().collect(),
        //FIXME: There has to be a way to make this a bit nicer looking
        "h" => s
            .replace(" ", "")
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|c| c.iter().collect::<String>())
            .map(|c| u8::from_str_radix(&c, 16).unwrap())
            .collect(),
        "f" => from_file(s),
        _ => {
            eprint!("unknown option -{opt}{line_end}");
            let msg = if env::consts::OS == "windows" {
                (*DOC).replace("\n", "\r\n")
            } else {
                DOC.to_string()
            };
            eprint!("{msg}{line_end}");
            exit(1)
        }
    }
}
