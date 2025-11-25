/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Character set definitions for the tool.
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::LazyLock;

use crate::error::XorError;
// FIXME: Generally, there's a lot here that could be cleaned up

/// Mapping of the short forms to sets of characters
static CHARSETS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("a", "abcdefghijklmnopqrstuvwxyz");
    m.insert("A", "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    m.insert("1", "0123456789");
    m.insert("!", "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~");
    // NOTE: Rust does not seem to support \v and \f escape characters, so we used \x0b and \x0c instead, as those are allowed
    m.insert("*", "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~ \t\n\r\x0b\x0c");
    m
});

/// Mapping of some pre-defined character sets.
pub static PREDEFINED_CHARSETS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("base32", "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=");
        m.insert(
            "base64",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/+=",
        );
        m.insert("printable", CHARSETS.get("*").unwrap());
        m
    });

/// Get a character from the short form combination
///
/// # Arguments
///   * `charset`: Either the name of a predefined charset to use, or
///     A set of characters for the to combine
///
/// # Returns
///   Created character set
///
/// # Errors
///   Returns `XorError::CharsetError` if an invalid letter is used for
///   building a charset.
pub fn get_charset(charset: &str) -> Result<String, XorError> {
    let charset = if charset.is_empty() {
        "printable"
    } else {
        charset
    };
    if PREDEFINED_CHARSETS.contains_key(charset) {
        return Ok((*PREDEFINED_CHARSETS.get(charset).unwrap()).to_owned());
    }

    let mut chars = String::new();
    for c in charset.chars() {
        if CHARSETS.contains_key(c.to_string().as_str()) {
            write!(chars, "{}", CHARSETS.get(c.to_string().as_str()).unwrap()).unwrap();
        } else {
            return Err(XorError::CharsetError { charset: c });
        }
    }
    Ok(chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_str() {
        assert_eq!(
            get_charset(""),
            Ok(PREDEFINED_CHARSETS["printable"].to_string())
        );
    }

    #[test]
    fn test_predefined_charsets() {
        for c in PREDEFINED_CHARSETS.keys() {
            assert_eq!(get_charset(c), Ok(PREDEFINED_CHARSETS[c].to_string()))
        }
    }

    #[test]
    fn test_building_charset() {
        assert_eq!(
            get_charset("aA"),
            Ok(String::from(
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            ))
        );
    }

    #[test]
    fn test_invalid_charset() {
        assert_eq!(
            get_charset("aZ"),
            Err(XorError::CharsetError { charset: 'Z' })
        )
    }
}
