/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Core logic for xortool
use crate::{
    VERSION,
    colors::{C_BEST_KEYLEN, C_BEST_PROB, C_FATAL, C_KEYLEN, C_PROB},
    error::XorError,
    routine::{dexor, mkdir},
};
use std::{
    ascii::escape_default,
    collections::{HashMap, hash_map::Entry},
    env,
    fmt::Write as _,
    fs,
    io::Write as _,
    path::MAIN_SEPARATOR,
    process::exit,
    sync::LazyLock,
};

/// Command line documentation for the tool
static DOC: LazyLock<String> = LazyLock::new(|| {
    format!("
xortool {VERSION}\
A tool to do some xor analysis:
- guess the key length (based on count of equal chars)
- guess the key (base on knowledge of most frequent char)

Usage:
xortool [-x] [-m MAXLEN] [-f] [-t CHARSET] [FILE]
xortool [-x] [-l LEN] [-c CHAR | -b | -o] [-f] [-t CHARSET] [-p PLAIN] [-r PERCENT] [FILE]
xortool [-x] [-m MAXLEN| -l LEN] [-c CHAR | -b | -o] [-f] [-t CHARSET] [-p PLAIN] [-r PERCENT] [FILE]
xortool [-h | --help]
xortool --version

Options:
-x --hex                           input is hex-encoded str
-l LEN, --key-length=LEN           length of the key
-m MAXLEN, --max-keylen=MAXLEN   maximum key length to probe [default: 65]
-c CHAR, --char=CHAR               most frequent char (one char or hex code)
-b --brute-chars                   brute force all possible most frequent chars
-o --brute-printable               same as -b but will only check printable chars
-f --filter-output                 filter outputs based on the charset
-t CHARSET --text-charset=CHARSET  target text character set [default: printable]
-p PLAIN --known-plaintext=PLAIN   use known plaintext for decoding
-r PERCENT, --threshold=PERCENT    threshold validity percentage [default: 95]
-h --help                          show this help

Notes:
Text character set:
    * Pre-defined sets: printable, base32, base64
    * Custom sets:
    - a: lowercase chars
    - A: uppercase chars
    - 1: digits
    - !: special chars
    - *: printable chars

Examples:
xortool file.bin
xortool -l 11 -c 20 file.bin
xortool -x -c ' ' file.hex
xortool -b -f -l 23 -t base64 message.enc
xortool -r 80 -p \"flag{{\" -c ' ' message.enc
")
});

// TODO: Support changing with CLI arg
/// Directory to put decrypted data in
const DIRNAME: &str = "xortool_out";

use crate::{
    args::{Parameters, parse_parameters},
    charset::PREDEFINED_CHARSETS,
    colors::{C_COUNT, C_DIV, C_KEY, C_RESET, C_WARN},
    routine::{decode_from_hex, die, load_file, rmdir},
};

/// Main function for xortool
///
/// # Arguments
///   * `args`: Optional vector of arguments to parse. If not supplied,
///     arguments are read from the command line instead.
pub fn main(args: Option<Vec<String>>) {
    let result = main_inner(args);
    let line_end = if env::consts::OS == "windows" {
        "\r\n"
    } else {
        "\n"
    };
    match result {
        Ok(()) => (),
        Err(e) => {
            print!("{}{e}{}{line_end}", *C_FATAL, *C_RESET);
            exit(1)
        }
    }
}

/// Inner logic of the main function for xortool
///
/// # Arguments
///   * `args`: Optional vector of arguments to parse. If not supplied,
///     arguments are read from the command line instead.
///
/// # Errors
///   Returns any errors that occurred during tool execution
fn main_inner(args: Option<Vec<String>>) -> Result<(), XorError> {
    let mut param = parse_parameters(&DOC, VERSION, args)?;
    let ciphertext = get_ciphertext(&param);
    if param.known_key_length.is_none() {
        param.known_key_length = Some(guess_key_length(&ciphertext, &param)?);
    }

    let try_chars: Vec<u8> = if param.brute_chars {
        (0..=255).collect()
    } else if param.brute_printable {
        PREDEFINED_CHARSETS
            .get("printable")
            .unwrap()
            .bytes()
            .collect()
    } else if param.most_frequent_char.is_some() {
        vec![param.most_frequent_char.unwrap()]
    } else {
        die(
            &format!(
                "{}Most possible char is needed to guess the key!{}",
                *C_WARN, *C_RESET
            ),
            1,
        );
        // This is never actually hit, as die() causes termination
        Vec::new()
    };

    let (probable_keys, key_char_used) =
        guess_probable_keys_for_chars(&ciphertext, &try_chars, &param);

    print_keys(&probable_keys);
    produce_plaintext(&ciphertext, &probable_keys, &key_char_used, &param)?;

    // FIXME: Need Exception handling. Needs to be bubbled up from functions instead of them panicking.
    // cleanup();
    Ok(())
}

/// Read in the encrypted data
///
/// # Arguments
///   * `param`: Command line parameters supplied to the tool
///
/// # Returns
///   The bytes of the encrypted data.
fn get_ciphertext(param: &Parameters) -> Vec<u8> {
    let ciphertext = load_file(&param.filename);

    if param.input_is_hex {
        return decode_from_hex(&ciphertext);
    }
    ciphertext
}

// -----------------------------------------------------------------------------
// KEYLENGTH GUESSING SECTION
// -----------------------------------------------------------------------------

/// Guess the length of the key used to encrypt the data
///
/// # Arguments
///   * `text`: The encrypted data
///   * `param`: Command line parameters provided to the tool
///
/// # Returns
///   The guessed length of the key used to encrypt the data.
///
/// # Error
///   Returns `AnalysisError` if no candidates could be found for key length.
fn guess_key_length(text: &[u8], param: &Parameters) -> Result<i32, XorError> {
    let mut fitnesses = calculate_fitnesses(text, param);
    if fitnesses.is_empty() {
        return Err(XorError::AnalysisError {
            msg: String::from("No candidates for key length found! Too small file?"),
        });
    }
    // Sorting here instead of inside the print_fitnesses function since
    // in Python, the list was passed by reference and thus sorted for all
    // later functions. But here we pass a immutable slice, so if we sorted
    // in the function, it would only apply to that function
    fitnesses.sort_by(|a, b| a.1.total_cmp(&b.1));
    fitnesses.reverse();

    print_fitnesses(&fitnesses);
    guess_and_print_divisors(&fitnesses, param);
    Ok(get_max_fitnessed_key_length(&fitnesses))
}

/// Calculate fitness of different key lengths
///
/// # Arguments
///   * `text`: The encrypted data to decode
///   * `param`: The command line parameters passed to the tool
///
/// # Returns
///   Vector of tuples. For each tuple, the first element is a key
///   length, and the second is the fitness of that key length
fn calculate_fitnesses(text: &[u8], param: &Parameters) -> Vec<(i32, f64)> {
    let mut prev = 0.0;
    let mut pprev = 0.0;
    let mut fitnesses = Vec::new();

    let max_key_len = param.max_key_length.unwrap_or_default();

    let range_end = match param.max_key_length {
        Some(i) => i + 1,
        None => 0,
    };

    let mut outer_key_len = 0;

    for key_length in 1..range_end {
        let fitness = f64::from(count_equals(text, key_length));

        let fitness = fitness / (f64::from(max_key_len) + (f64::from(key_length)).powf(1.5));

        if pprev < prev && prev > fitness {
            // Local maximum
            fitnesses.push((key_length - 1, prev));
        }

        pprev = prev;
        prev = fitness;
        outer_key_len = key_length;
    }

    if pprev < prev {
        fitnesses.push((outer_key_len - 1, prev));
    }

    fitnesses
}

/// Pint out top 10 key lengths by fitness
///
/// # Argument
///   * `fitnesses`: Slice of tuples of the fitnesses. First element in tuple
///     is key length. Second is fitness as a float.
fn print_fitnesses(fitnesses: &[(i32, f64)]) {
    let line_end = if env::consts::OS == "windows" {
        "\r\n"
    } else {
        "\n"
    };
    print!("The most probable key lengths:{line_end}");

    // Top sorted by fitness, but print sorted by length.
    // NOTE: Original Python had sorting here, but we moved it to outer
    // function. See the outer function for a comment on why

    let mut top10: Vec<(i32, f64)> = fitnesses.iter().take(10).copied().collect();
    let best_fitness = top10[0].1;
    top10.sort_by_key(|v| v.0);

    let fitness_sum = calc_fitness_sum(&top10);
    // FIXME: Can we do this without string formatting?
    let largest_number = top10.iter().map(|v| v.0).max().unwrap();
    let largest_width = format!("{largest_number}").len();

    for (key_length, fitness) in top10 {
        let pct = 100.0 * fitness * 1.0 / fitness_sum;
        if fitness == best_fitness {
            print!(
                "{}{key_length:>width$}{}: {}{pct:5.1}%{}{line_end}",
                *C_BEST_KEYLEN,
                *C_RESET,
                *C_BEST_PROB,
                *C_RESET,
                width = largest_width
            );
        } else {
            print!(
                "{}{key_length:>width$}{}: {}{pct:5.1}%{}{line_end}",
                *C_KEYLEN,
                *C_RESET,
                *C_PROB,
                *C_RESET,
                width = largest_width
            );
        }
    }
}

/// Compute the sum of all of the fitnesses
///
/// # Arguments
///   * `fitnesses`: The fitnesses to sum
///
/// # Returns
///   Sum of all of the fitnesses
fn calc_fitness_sum(fitnesses: &[(i32, f64)]) -> f64 {
    // FIXME: Probably a better way to do this
    let mut sum = 0.0;
    for (_, val) in fitnesses {
        sum += val;
    }
    sum
}

/// Count number of equal characters at all offsets up to `key_length` and sum
///
/// # Arguments
///   * `text`: The text to count the characters of
///   * `key_length`: The length of the key used to encrypt the data
///
/// # Returns
///   Sum of the counts of most common character at each offset up to `key_length`
fn count_equals(text: &[u8], key_length: i32) -> i32 {
    let mut equals_count = 0;
    if usize::try_from(key_length).unwrap() >= text.len() {
        return 0;
    }

    for offset in 0..key_length {
        let chars_count = chars_count_at_offset(text, key_length, offset);
        equals_count += chars_count.values().max().unwrap() - 1;
    }
    equals_count
}

/// Guess and print common divisions and return the most common divisor
///
/// # Arguments
///   * `fitnesses`: Slice of tuples of (key length, fitness)
///
/// # Returns
///   The most common divisor.
fn guess_and_print_divisors(fitnesses: &[(i32, f64)], param: &Parameters) -> i32 {
    let line_end = if env::consts::OS == "windows" {
        "\r\n"
    } else {
        "\n"
    };
    let max_key_len = param.max_key_length.unwrap_or_default();

    let mut divisors_counts = Vec::from([0]).repeat(usize::try_from(max_key_len).unwrap() + 1);
    for &(key_length, _) in fitnesses {
        for number in 3..=key_length {
            if key_length % number == 0 {
                divisors_counts[usize::try_from(number).unwrap()] += 1;
            }
        }
    }
    let max_divisors = divisors_counts.iter().max().unwrap();

    let mut limit = 3;
    let mut ret = 2;
    for (number, divisors_count) in divisors_counts.iter().enumerate() {
        if divisors_count == max_divisors {
            print!(
                "Key-length can be {}{}*n{}{line_end}",
                *C_DIV, number, *C_RESET
            );
            ret = number;
            limit -= 1;
            if limit == 0 {
                return ret.try_into().unwrap();
            }
        }
    }
    ret.try_into().unwrap()
}

/// Get the key length that has the highest fitness
///
/// # Arguments
///   * `fitnesses`: Slice of tuples of (key length, fitness)
///
/// # Returns
///   The key length that has the highest fitness.
fn get_max_fitnessed_key_length(fitnesses: &[(i32, f64)]) -> i32 {
    let mut max_fitness = 0.0;
    let mut max_fitnessed_key_length = 0;
    for &(key_length, fitness) in fitnesses {
        if fitness > max_fitness {
            max_fitness = fitness;
            max_fitnessed_key_length = key_length;
        }
    }
    max_fitnessed_key_length
}

/// Count occurrences of characters starting at `offset` every `key_length`
///
/// Starting at the index `offset`, for ever `key_length` characters, the
/// value of a character is read and recorded. The total counts of found
/// characters are then returned.
///
/// # Arguments
///   * `text`: Data to count characters of
///   * `key_length`: The length of the key used to encrypt the data
///   * `offset`: Offset to start point for counting characters
///
/// # Returns
///
///  `HashMap` where the keys are characters found in the data set, and the
///   values are the number of occurrences of the character.
fn chars_count_at_offset(text: &[u8], key_length: i32, offset: i32) -> HashMap<u8, i32> {
    let mut chars_count = HashMap::new();
    for pos in
        (usize::try_from(offset).unwrap()..text.len()).step_by(usize::try_from(key_length).unwrap())
    {
        let c = text[pos];

        match chars_count.entry(c) {
            Entry::Vacant(e) => e.insert(1),
            Entry::Occupied(e) => {
                let tmp_ref = e.into_mut();
                *tmp_ref += 1;
                tmp_ref
            }
        };
    }
    chars_count
}

// -----------------------------------------------------------------------------
// KEYS GUESSING SECTION
// -----------------------------------------------------------------------------

/// Guess probably keys for all of a list of possible most common characters
///
/// # Arguments
///   * `text`: The encrypted data
///   * `try_chars`: Characters to try as the most common character.
///   * `param`: Command line parameters supplied to the tool
///
/// # Returns
///   * Vector of Vectors, where each inner vector is the bytes of a probable key
///   * `HashMap` that maps the probable keys to the most common char they were found by.
fn guess_probable_keys_for_chars(
    text: &[u8],
    try_chars: &[u8],
    param: &Parameters,
) -> (Vec<Vec<u8>>, HashMap<Vec<u8>, u8>) {
    let mut probable_keys = Vec::new();
    let mut key_char_used = HashMap::new();
    for c in try_chars {
        let keys = guess_keys(text, *c, param);
        for key in keys {
            key_char_used.insert(key.clone(), *c);
            if !probable_keys.contains(&key) {
                probable_keys.push(key);
            }
        }
    }
    (probable_keys, key_char_used)
}

/// Guess keys for the given text, based on the known most frequent character
///
/// # Arguments:
///   * `text`: The encrypted data
///   * `most_char`: The most common character in the decrypted data
///   * `param`: Command line parameters supplied to the tool.
///
/// # Returns
///   Vector of vectors of bytes for possible keys
fn guess_keys(text: &[u8], most_char: u8, param: &Parameters) -> Vec<Vec<u8>> {
    let key_length = param.known_key_length.unwrap_or_default();

    let mut key_possible_bytes = Vec::new();
    for _ in 0..key_length {
        key_possible_bytes.push(Vec::new());
    }

    for offset in 0..key_length {
        let chars_count = chars_count_at_offset(text, key_length, offset);
        let max_count = *chars_count.values().max().unwrap();
        for (character, count) in chars_count {
            if count >= max_count {
                key_possible_bytes[usize::try_from(offset).unwrap()].push(character ^ most_char);
            }
        }
    }
    all_keys(&key_possible_bytes, &[], 0)
}

/// Product all combinations of possible key chars
///
/// # Arguments
///   * `key_possible_bytes`: Vector of vectors, where each sub-vector
///     is a set of characers used in a key
///   * `key_part`: Portion of a possible key
///   * `offset`: Offset into `key_possible_bytes` to loop over.
///
/// # Warning
///   This function is recursive
///
/// # Returns
///   Vector of vectors of the possible key combinations
fn all_keys(key_possible_bytes: &Vec<Vec<u8>>, key_part: &[u8], offset: usize) -> Vec<Vec<u8>> {
    let mut keys = Vec::new();
    if offset >= key_possible_bytes.len() {
        return Vec::from([key_part.to_vec()]);
    }
    for c in &key_possible_bytes[offset] {
        let mut tmp = key_part.to_vec();
        tmp.push(*c);
        keys.extend(all_keys(key_possible_bytes, &tmp, offset + 1));
    }
    keys
}

/// Print out all of the keys that the tool has guessed
///
/// # Arguments
///   * `keys`: The keys that the tool has guessed.
fn print_keys(keys: &[Vec<u8>]) {
    let line_end = if env::consts::OS == "windows" {
        "\r\n"
    } else {
        "\n"
    };
    if keys.is_empty() {
        print!("No keys guessed!{line_end}");
        return;
    }
    print!(
        "{}{}{} possible key(s) of length {}{}{}:{line_end}",
        *C_COUNT,
        keys.len(),
        *C_RESET,
        *C_COUNT,
        keys[0].len(),
        *C_RESET
    );

    for key in keys.iter().take(5) {
        print!("{}{}{}{line_end}", *C_KEY, to_printable_key(key), *C_RESET);
    }
    if keys.len() > 10 {
        print!("...{line_end}");
    }
}

/// Convert a key into printable format
///
/// # Arguments
///   * `bytes`: The bytes of the key to compute
///
/// # Returns
///   The key in a printable/displayable format.
fn to_printable_key(bytes: &[u8]) -> String {
    let mut result = String::new();
    for &byte in bytes {
        for c in escape_default(byte) {
            result.push(c as char);
        }
    }
    // To match the original test, we don't want to escape the quote character.
    let result = result.replace("\\\"", "\"");
    if result.contains('\'') && !result.contains('"') {
        result.replace("\\'", "'")
    } else {
        result
    }
}

// -----------------------------------------------------------------------------
// RETURNS PERCENTAGE OF VALID TEXT CHARS
// -----------------------------------------------------------------------------
/// Calculate the percentage of characters in the text that are within the charset in use
///
/// # Arguments
///   * `text`: The text to check
///   * `param`: The parameters to use
///
/// # Returns
///   Percentage of characters in `text` that are within the charset
fn percentage_valid(text: &[u8], param: &Parameters) -> f64 {
    let mut x = 0.0;
    for c in text {
        if param.text_charset.contains(c) {
            x += 1.0;
        }
    }
    x / (text.len() as f64)
}

// -----------------------------------------------------------------------------
// PRODUCE OUTPUT
// -----------------------------------------------------------------------------

/// Produce plaintext variant for each possible key,
/// creates csv files with keys, percentage of valid
/// characters and used most frequent character
fn produce_plaintext(
    ciphertext: &[u8],
    keys: &[Vec<u8>],
    key_char_used: &HashMap<Vec<u8>, u8>,
    param: &Parameters,
) -> Result<(), XorError> {
    cleanup();
    mkdir(DIRNAME)?;

    let line_end = if env::consts::OS == "windows" {
        "\r\n"
    } else {
        "\n"
    };

    // this is split up in two files since the
    // key can contain all kinds of characters
    let fn_key_mapping = "filename-key.csv";
    let fn_perc_mapping = "filename-char_used-perc_valid.csv";

    let mut key_mapping = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{DIRNAME}{MAIN_SEPARATOR}{fn_key_mapping}"))
        .unwrap();
    let mut perc_mapping = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{DIRNAME}{MAIN_SEPARATOR}{fn_perc_mapping}"))
        .unwrap();

    key_mapping
        .write_fmt(format_args!("file_name;key_repr{line_end}"))
        .unwrap();
    perc_mapping
        .write_fmt(format_args!("file_name;char_used;perc_valid{line_end}"))
        .unwrap();

    let threshold_valid = param.threshold.unwrap_or(95);

    let mut count_valid = 0;

    for (index, key) in keys.iter().enumerate() {
        let key_index = format!(
            "{index:0>width$}",
            width = format!("{}", (keys.len() - 1)).len(),
        );
        // FIXME: SHould be repr(key) in python
        let key_repr = to_printable_key(key);
        let file_name = format!("{DIRNAME}{MAIN_SEPARATOR}{key_index}.out");

        let dexored = dexor(ciphertext, key);
        // ignore saving file when known plain is provided and output doesn't contain it
        if !param.known_plain.is_empty()
            && !dexored
                .windows(param.known_plain.len())
                .collect::<Vec<&[u8]>>()
                .contains(&param.known_plain.as_slice())
        {
            continue;
        }
        let perc = (100.0 * percentage_valid(&dexored, param)).round() as i32;
        if perc > threshold_valid {
            count_valid += 1;
        }
        // FIXME: write(format) vs write_fmt(format_args)
        if key_repr.contains('\'') && !key_repr.contains("\\'") {
            key_mapping
                .write_all(format!("{file_name};b\"{key_repr}\"{line_end}").as_bytes())
                .unwrap();
        } else {
            key_mapping
                .write_all(format!("{file_name};b'{key_repr}'{line_end}").as_bytes())
                .unwrap();
        }
        // FIXME: SHould be repr(key_char_used[key])
        perc_mapping
            .write_fmt(format_args!(
                "{file_name};{:?};{perc}{line_end}",
                key_char_used[key]
            ))
            .unwrap();
        if !param.filter_output || (perc > threshold_valid) {
            fs::write(file_name, dexored).unwrap();
        }
    }

    let mut msg = format!(
        "Found {}{count_valid}{} plaintexts with {}{threshold_valid}{}%+ valid characters",
        *C_COUNT, *C_RESET, *C_COUNT, *C_RESET
    );
    if !param.known_plain.is_empty() {
        write!(
            msg,
            " which contained '{}'",
            String::from_utf8(param.known_plain.clone()).unwrap()
        )
        .unwrap();
    }
    print!("{msg}{line_end}");
    print!("See files {fn_key_mapping}, {fn_perc_mapping}{line_end}");

    Ok(())
}

// FIXME: Make this smarter/safer?
/// Delete the output directory if it already exists.
fn cleanup() {
    if fs::exists(DIRNAME).unwrap() {
        rmdir(DIRNAME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ciphertext() {
        let param = Parameters {
            filename: String::from("tests/small_file.txt"),
            ..Default::default()
        };
        assert_eq!(get_ciphertext(&param), "Hello World!".as_bytes())
    }

    #[test]
    fn test_get_ciphertext_hex() {
        let param = Parameters {
            filename: String::from("tests/small_file_hex.txt"),
            input_is_hex: true,
            ..Default::default()
        };
        assert_eq!(get_ciphertext(&param), "Hello World".as_bytes())
    }

    #[test]
    fn test_calc_fitness_sum() {
        let fitnesses = [(1, 3.2), (5, 8.3), (7, 9.3)];
        assert_eq!(calc_fitness_sum(&fitnesses), 20.8);
    }

    #[test]
    fn test_count_equals() {
        let text = "Hello World!".as_bytes();
        assert_eq!(count_equals(text, 2), 1)
    }

    #[test]
    fn test_count_equals_large_key() {
        assert_eq!(count_equals("Hi".as_bytes(), 4), 0);
    }

    #[test]
    fn test_get_max_fitnessed_key_length() {
        let fitnesses = [(1, 3.2), (5, 18.3), (7, 9.3)];
        assert_eq!(get_max_fitnessed_key_length(&fitnesses), 5);
    }

    #[test]
    fn test_chars_count_at_offset() {
        let text = "Hello World!".as_bytes();
        let mut expected = HashMap::new();
        expected.insert(' ' as u8, 1);
        expected.insert('l' as u8, 2);
        expected.insert('o' as u8, 1);
        expected.insert('!' as u8, 1);
        assert_eq!(chars_count_at_offset(text, 2, 3), expected)
    }

    #[test]
    fn test_percentage_valid() {
        let p = Parameters {
            text_charset: vec!['a' as u8, 'b' as u8, 'c' as u8],
            ..Default::default()
        };
        let text = "hela abc";
        assert_eq!(percentage_valid(text.as_bytes(), &p), 0.5)
    }
}
