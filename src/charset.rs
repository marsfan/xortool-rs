/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::error::XorError;
// FIXME: Generally, there's a lot here that could be cleaned up

lazy_static! {

    static ref CHARSETS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("a", "abcdefghijklmnopqrstuvwxyz");
        m.insert("A", "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        m.insert("1", "0123456789");
        m.insert("!", "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~");
        // NOTE: Rust does not seem to support \v and \f escape characters, so we used \x0b and \x0c instead, as those are allowed
        m.insert("*", "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~ \t\n\r\x0b\x0c");
        m
    };

    pub static ref PREDEFINED_CHARSETS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("base32","ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=");
        m.insert("base64","abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/+=");
        m.insert("printable", CHARSETS.get("*").unwrap());
        m
    };

}

pub fn get_charset(charset: &str) -> Result<String, XorError> {
    let charset = if charset == "" { "printable" } else { charset };
    if PREDEFINED_CHARSETS.contains_key(charset) {
        return Ok(PREDEFINED_CHARSETS.get(charset).unwrap().to_string());
    }

    let mut chars = String::new();
    for c in charset.chars() {
        if (*CHARSETS).contains_key(c.to_string().as_str()) {
            chars.push_str(CHARSETS.get(c.to_string().as_str()).unwrap());
        } else {
            return Err(XorError::CharsetError { charset: c });
        }
    }
    return Ok(chars);
}
