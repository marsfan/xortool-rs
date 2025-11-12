use std::{fs, io::Read, panic::UnwindSafe, path::PathBuf, str::Bytes};

pub fn load_file(filename: &str) -> Vec<u8> {
    if filename == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).unwrap();
        println!("{}", String::from_utf8(buf.clone()).unwrap());
        return buf;
    }
    std::fs::read(filename).unwrap()
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

pub fn decode_from_hex(text: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(text);
    let mut only_hex_digits = String::new();
    for character in text.chars() {
        if "0123456789abcdefABCDEF".contains(character) {
            only_hex_digits.push_str(&character.to_string());
        }
    }
    only_hex_digits.bytes().collect()
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
