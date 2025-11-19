/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use std::{fs, io::Read};

use crate::error::XorError;

pub fn load_file(filename: &str) -> Vec<u8> {
    if filename == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).unwrap();
        return buf;
    }
    std::fs::read(filename).unwrap()
}

pub fn mkdir(dirname: &str) -> Result<(), XorError> {
    if fs::exists(&dirname).unwrap() {
        return Ok(());
    } else {
        match fs::create_dir(&dirname) {
            Ok(_) => Ok(()),
            Err(e) => Err(XorError::MkdirError { msg: e.to_string() }),
        }
    }
}

pub fn rmdir(dirname: &str) {
    let metadata = fs::symlink_metadata(&dirname).unwrap();
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return; // do not clear link - we can get out of dir
    }
    fs::remove_dir_all(dirname).unwrap();
}

pub fn decode_from_hex(text: &[u8]) -> Vec<u8> {
    // FIXME: Can probably make this a lot cleaner
    let mut only_hex_digits = Vec::new();
    for &character in text {
        if "0123456789abcdefABCDEF".as_bytes().contains(&character) {
            only_hex_digits.push(character);
        }
    }
    let mut result = Vec::new();
    assert_eq!(only_hex_digits.len() % 2, 0);
    for chunk in only_hex_digits.chunks_exact(2) {
        let chunk_str = String::from_utf8(chunk.to_vec()).unwrap();

        result.push(u8::from_str_radix(&chunk_str, 16).unwrap());
    }
    result
}

pub fn dexor(text: &[u8], key: &[u8]) -> Vec<u8> {
    let val_mod = key.len();
    let mut results = Vec::new();
    for (index, char) in text.iter().enumerate() {
        let tmp = key[index % val_mod] ^ char;
        results.push(tmp);
    }
    results
}

pub fn die(exit_message: String, exit_code: i32) {
    println!("{exit_message}");
    std::process::exit(exit_code);
}

// `is_linux` and `alphanum` in the original source are dead code.
// never used anywhere, so I just removed them.
