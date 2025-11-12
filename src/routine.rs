use std::{fs, io::Read, panic::UnwindSafe, path::PathBuf, str::Bytes};

pub fn load_file(filename: &str) -> String {
    if filename == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap();
        return buf;
    }
    std::fs::read_to_string(filename).unwrap()
}

pub fn save_file(filename: String, data: &[u8]) {
    std::fs::write(filename, data).unwrap()
}

pub fn mkdir(dirname: &str) {
    if fs::exists(&dirname).unwrap() {
        return;
    } else {
        fs::create_dir(&dirname).unwrap();
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

pub fn decode_from_hex(text: &str) -> Vec<u8> {
    // FIXME: Can probably make this a lot cleaner
    let mut only_hex_digits = String::new();
    for character in text.chars() {
        if "0123456789abcdefABCDEF".contains(character) {
            only_hex_digits.push_str(&character.to_string());
        }
    }
    let mut result = Vec::new();
    assert_eq!(only_hex_digits.len() % 2, 0);
    for chunk in only_hex_digits
        .chars()
        .collect::<Vec<char>>()
        .chunks_exact(2)
    {
        assert_eq!(chunk.len(), 2);
        let s = chunk.iter().collect::<String>();
        result.push(u8::from_str_radix(&s, 16).unwrap());
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
