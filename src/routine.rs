/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Various routines used by the tool
use std::{env, fs, io, io::Read as _, process::exit};

use crate::error::XorError;

/// Load from a file (or stdin)
///
/// # Arguments
///   * `filename`: The name of the file to load from, or `-` to load
///     from standard input
///
/// # Returns
///   Vector of the bytes read from the file, or standard input.
pub fn load_file(filename: &str) -> Vec<u8> {
    if filename == "-" {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap();
        return buf;
    }
    fs::read(filename).unwrap()
}

/// Create directory with the given name
///
/// # Arguments
///   * `dirname`: The name of the directory to create
///
/// # Error
///   creates `XorError::MkdirError` if creating the directory failed
pub fn mkdir(dirname: &str) -> Result<(), XorError> {
    if fs::exists(dirname).unwrap() {
        Ok(())
    } else {
        match fs::create_dir(dirname) {
            Ok(()) => Ok(()),
            Err(e) => Err(XorError::MkdirError { msg: e.to_string() }),
        }
    }
}

/// Delete the given directory
///
/// # Arguments
///   * `dirname`: The name of the directory to delete
pub fn rmdir(dirname: &str) {
    let metadata = fs::symlink_metadata(dirname).unwrap();
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return; // do not clear link - we can get out of dir
    }
    fs::remove_dir_all(dirname).unwrap();
}

/// Decode a string of hexadecimal values into their values
///
/// This takes input text that is hex values (e.g. "01 3D DE AD BE EF")
/// and parses the hex values into their character equivlents.
///
/// # Arguments
///   * `text`: The bytes of the text to decode
///
/// # Returns
///   Vector of the bytes of the decoded text.
pub fn decode_from_hex(text: &[u8]) -> Vec<u8> {
    // FIXME: Can probably make this a lot cleaner
    let mut only_hex_digits = Vec::new();
    for &character in text {
        if "0123456789abcdefABCDEF".as_bytes().contains(&character) {
            only_hex_digits.push(character);
        }
    }
    let mut result = Vec::new();
    assert_eq!(
        only_hex_digits.len() % 2,
        0,
        "Odd number of characters after extracting only hex digits."
    );
    for chunk in only_hex_digits.chunks_exact(2) {
        let chunk_str = String::from_utf8(chunk.to_vec()).unwrap();

        result.push(u8::from_str_radix(&chunk_str, 16).unwrap());
    }
    result
}

/// Reverse xor encryption on a set of bytes
///
/// # Arguments
///   * `text`: The xor-encoded data
///   * `key`: The key used to encrypt the data
///
/// # Returns
///   Decrypted bytes
pub fn dexor(text: &[u8], key: &[u8]) -> Vec<u8> {
    let val_mod = key.len();
    let mut results = Vec::new();
    for (index, char) in text.iter().enumerate() {
        let tmp = key[index % val_mod] ^ char;
        results.push(tmp);
    }
    results
}

/// Exit the program and display the given error message
///
/// # Arguments
///   * `exit_message`: The message to display on exit.
///   * `exit_code`: The exit code to exit the program with.
pub fn die(exit_message: &str, exit_code: i32) {
    if env::consts::OS == "windows" {
        println!("{exit_message}\r");
    } else {
        println!("{exit_message}");
    }
    exit(exit_code);
}

// `is_linux` and `alphanum` in the original source are dead code.
// never used anywhere, so I just removed them.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_file() {
        assert_eq!(load_file("tests/small_file.txt"), "Hello World!".as_bytes());
    }

    #[test]
    fn test_mkdir_already_exists() {
        assert_eq!(mkdir("src"), Ok(()))
    }

    #[test]
    fn test_mkdir_error() {
        let result = mkdir("src/hello/world");
        // Exact message is platform specific, so just check to make sure the right error type is created.
        match result {
            Ok(_) => assert!(false),
            Err(e) => match e {
                XorError::MkdirError { msg: _ } => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_decode_from_hex() {
        let input = "48 65 6C 6C 6F 20 57 6F 72 6C 64".as_bytes();
        assert_eq!(decode_from_hex(input), "Hello World".as_bytes());
    }

    #[test]
    fn test_dexor() {
        let text = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let key = vec![3, 2, 1];
        assert_eq!(dexor(&text, &key), vec![2, 0, 2, 7, 7, 7, 4, 10]);
    }
}
