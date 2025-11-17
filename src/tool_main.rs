use lazy_static::lazy_static;

use crate::{
    VERSION,
    colors::{C_BEST_KEYLEN, C_BEST_PROB, C_KEYLEN, C_PROB},
    routine::{dexor, mkdir},
};
use std::{ascii::escape_default, collections::HashMap, fs, io::Write, path::MAIN_SEPARATOR};
lazy_static! {


    static ref DOC: String = format!("
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
");

}
const DIRNAME: &str = "xortool_out";

use crate::{
    args::{Parameters, parse_parameters},
    charset::PREDEFINED_CHARSETS,
    colors::{C_COUNT, C_DIV, C_KEY, C_RESET, C_WARN},
    routine::{decode_from_hex, die, load_file, rmdir},
};

pub fn main() {
    let mut param = parse_parameters(&DOC, VERSION);
    let ciphertext = get_ciphertext(&param);
    if param.known_key_length.is_none() {
        param.known_key_length = Some(guess_key_length(&ciphertext, &param))
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
        vec![param.most_frequent_char.unwrap().try_into().unwrap()]
    } else {
        die(
            format!(
                "{}Most possible char is needed to guess the key!{}",
                C_WARN.to_string(),
                C_RESET.to_string()
            ),
            1,
        );
        // This is never actually hit, as die() causes termination
        Vec::new()
    };

    let (probable_keys, key_char_used) =
        guess_probable_keys_for_chars(&ciphertext, &try_chars, &param);

    print_keys(&probable_keys);
    produce_plaintext(&ciphertext, &probable_keys, &key_char_used, &param);

    // FIXME: Need Exception handling. Needs to be bubbled up from functions instead of them panicking.
    // cleanup();
}

/// Loading ciphertext
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

fn guess_key_length(text: &[u8], param: &Parameters) -> i32 {
    let mut fitnesses = calculate_fitnesses(text, param);
    if fitnesses.is_empty() {
        panic!("No candidates for key length found! Too small file?");
    }
    // Sorting here instead of inside the print_fitnesses function since
    // in Python, the list was passed by reference and thus sorted for all
    // later functions. But here we pass a immutable slice, so if we sorted
    // in the function, it would only apply to that function
    fitnesses.sort_by(|a, b| a.1.total_cmp(&b.1));
    fitnesses.reverse();

    print_fitnesses(&fitnesses);
    guess_and_print_divisors(&fitnesses, param);
    get_max_fitnessed_key_length(&fitnesses)
}

fn calculate_fitnesses(text: &[u8], param: &Parameters) -> Vec<(i32, f64)> {
    let mut prev = 0.0;
    let mut pprev = 0.0;
    let mut fitnesses = Vec::new();

    let max_key_len = match param.max_key_length {
        Some(i) => i,
        None => 0,
    };
    let range_end = match param.max_key_length {
        Some(i) => i + 1,
        None => 0,
    };

    let mut outer_key_len = 0;

    for key_length in 1..range_end {
        let fitness = count_equals(text, key_length) as f64;

        let fitness = fitness / (max_key_len as f64 + (key_length as f64).powf(1.5));

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

fn print_fitnesses(fitnesses: &[(i32, f64)]) {
    println!("The most probable key lengths:");

    // Top sorted by fitness, but print sorted by length.
    // NOTE: Original Python had sorting here, but we moved it to outer
    // function. See the outer function for a comment on why

    let mut top10: Vec<(i32, f64)> = fitnesses.iter().take(10).map(|v| v.clone()).collect();
    let best_fitness = top10[0].1;
    top10.sort_by_key(|v| v.0);

    let fitness_sum = calc_fitness_sum(&top10);
    // FIXME: Can we do this without string formatting?
    let largest_number = top10.iter().map(|v| v.0).max().unwrap();
    let largest_width = format!("{largest_number}").len();

    for (key_length, fitness) in top10 {
        let pct = 100.0 * fitness * 1.0 / fitness_sum;
        if fitness == best_fitness {
            println!(
                "{}{key_length:>width$}{}: {}{pct:5.1}%{}",
                *C_BEST_KEYLEN,
                *C_RESET,
                *C_BEST_PROB,
                *C_RESET,
                width = largest_width
            );
        } else {
            println!(
                "{}{key_length:>width$}{}: {}{pct:5.1}%{}",
                *C_KEYLEN,
                *C_RESET,
                *C_PROB,
                *C_RESET,
                width = largest_width
            );
        }
    }
}

fn calc_fitness_sum(fitnesses: &[(i32, f64)]) -> f64 {
    // FIXME: Probably a better way to do this
    let mut sum = 0.0;
    for (_, val) in fitnesses {
        sum += val;
    }
    sum
}

///Count equal chars count for each offset and sum them
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

fn guess_and_print_divisors(fitnesses: &[(i32, f64)], param: &Parameters) -> i32 {
    let max_key_len = match param.max_key_length {
        Some(i) => i,
        None => 0,
    };
    let mut divisors_counts = Vec::from([0]).repeat(usize::try_from(max_key_len).unwrap() + 1);
    for (key_length, _) in fitnesses {
        for number in 3..(key_length + 1) {
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
            println!("Key-length can be {}{}*n{}", *C_DIV, number, *C_RESET);
            ret = number;
            limit -= 1;
            if limit == 0 {
                return ret.try_into().unwrap();
            }
        }
    }
    ret.try_into().unwrap()
}

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

fn chars_count_at_offset(text: &[u8], key_length: i32, offset: i32) -> HashMap<u8, i32> {
    let mut chars_count = HashMap::new();
    for pos in
        (usize::try_from(offset).unwrap()..text.len()).step_by(usize::try_from(key_length).unwrap())
    {
        let c = text[pos];
        if chars_count.contains_key(&c) {
            let tmp_ref = chars_count.get_mut(&c).unwrap();
            *tmp_ref += 1;
        } else {
            chars_count.insert(c, 1);
        }
    }
    chars_count
}

// -----------------------------------------------------------------------------
// KEYS GUESSING SECTION
// -----------------------------------------------------------------------------

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

fn guess_keys(text: &[u8], most_char: u8, param: &Parameters) -> Vec<Vec<u8>> {
    let key_length = match param.known_key_length {
        Some(i) => i,
        None => 0,
    };
    let mut key_possible_bytes = Vec::new();
    for _ in 0..key_length {
        key_possible_bytes.push(Vec::new());
    }

    for offset in 0..key_length {
        let chars_count = chars_count_at_offset(text, key_length, offset);
        let max_count = *chars_count.values().max().unwrap();
        for &character in chars_count.keys() {
            if chars_count[&character] >= max_count {
                key_possible_bytes[usize::try_from(offset).unwrap()].push(character ^ most_char);
            }
        }
    }
    all_keys(&key_possible_bytes, &[], 0)
}

fn all_keys(key_possible_bytes: &Vec<Vec<u8>>, key_part: &[u8], offset: usize) -> Vec<Vec<u8>> {
    let mut keys = Vec::new();
    if offset >= key_possible_bytes.len() {
        return Vec::from([key_part.to_vec()]);
    }
    for c in &key_possible_bytes[offset] {
        let mut tmp = key_part.to_vec();
        tmp.push(*c);
        keys.extend(all_keys(&key_possible_bytes, &tmp, offset + 1));
    }
    keys
}

fn print_keys(keys: &Vec<Vec<u8>>) {
    if keys.len() == 0 {
        println!("No keys guessed!");
        return;
    }
    println!(
        "{}{}{} possible key(s) of length {}{}{}:",
        C_COUNT.to_string(),
        keys.len(),
        C_RESET.to_string(),
        C_COUNT.to_string(),
        keys[0].len(),
        C_RESET.to_string()
    );

    for key in keys.iter().take(5) {
        println!(
            "{}{}{}",
            C_KEY.to_string(),
            to_printable_key(key),
            C_RESET.to_string()
        );
    }
    if keys.len() > 10 {
        println!("...");
    }
}

fn to_printable_key(bytes: &[u8]) -> String {
    let mut result = String::new();
    for &byte in bytes {
        for c in escape_default(byte) {
            result.push(c as char);
        }
    }
    // To match the original test, we don't want to escape the quote character.
    result.replace("\\\"", "\"").replace("\\'", "'")
}

// -----------------------------------------------------------------------------
// RETURNS PERCENTAGE OF VALID TEXT CHARS
// -----------------------------------------------------------------------------
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
    keys: &Vec<Vec<u8>>,
    key_char_used: &HashMap<Vec<u8>, u8>,
    param: &Parameters,
) {
    cleanup();
    mkdir(DIRNAME);

    // this is split up in two files since the
    // key can contain all kinds of characters
    let fn_key_mapping = "filename-key.csv";
    let fn_perc_mapping = "filename-char_used-perc_valid.csv";

    let mut key_mapping = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{DIRNAME}{MAIN_SEPARATOR}{fn_key_mapping}"))
        .unwrap();
    let mut perc_mapping = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{DIRNAME}{MAIN_SEPARATOR}{fn_perc_mapping}"))
        .unwrap();

    key_mapping
        .write_fmt(format_args!("file_name;key_repr\n"))
        .unwrap();
    perc_mapping
        .write_fmt(format_args!("file_name;char_used;perc_valid\n"))
        .unwrap();

    let threshold_valid = match param.threshold {
        Some(t) => t,
        None => 95,
    };

    let mut count_valid = 0;

    for (index, key) in keys.iter().enumerate() {
        let key_index = format!(
            "{index:0<width$}",
            width = format!("{}", (keys.len() - 1)).len(),
        );
        // FIXME: SHould be repr(key) in python
        let key_repr = format!("{}", to_printable_key(&key));
        let file_name = format!("{DIRNAME}{MAIN_SEPARATOR}{key_index}.out");

        let dexored = dexor(ciphertext, key);
        // ignore saving file when known plain is provided and output doesn't contain it
        if param.known_plain.len() != 0
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
        key_mapping
            .write_all(format!("{file_name};{key_repr}").as_bytes())
            .unwrap();
        // FIXME: SHould be repr(key_char_used[key])
        perc_mapping
            .write_fmt(format_args!("{file_name};{:?};{perc}", key_char_used[key]))
            .unwrap();
        if !param.filter_output || (param.filter_output && perc > threshold_valid) {
            fs::write(file_name, dexored).unwrap();
        }
    }

    let mut msg = format!(
        "Found {}{count_valid}{} plaintexts with {}{threshold_valid}{}%+ valid characters",
        *C_COUNT, *C_RESET, *C_COUNT, *C_RESET
    );
    if !param.known_plain.is_empty() {
        msg.push_str(&format!(
            " which contained '{}'",
            String::from_utf8(param.known_plain.clone()).unwrap()
        ));
    }
    println!("{msg}");
    println!("See files {fn_key_mapping}, {fn_perc_mapping}");
}

fn cleanup() {
    if std::fs::exists(DIRNAME).unwrap() {
        rmdir(DIRNAME);
    }
}
