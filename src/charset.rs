use std::collections::HashMap;

use lazy_static::lazy_static;
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

// FIXME: Proper Error Type
pub fn get_charset(charset: &str) -> Result<String, ()> {
    let charset = if charset == "" { "printable" } else { charset };
    if PREDEFINED_CHARSETS.contains_key(charset) {
        return Ok(PREDEFINED_CHARSETS.get(charset).unwrap().to_string());
    }

    let mut chars = String::new();
    for c in charset.chars() {
        if (*CHARSETS).contains_key(c.to_string().as_str()) {
            chars.push_str(CHARSETS.get(c.to_string().as_str()).unwrap());
        } else {
            return Err(());
        }
    }
    return Ok(chars);
}
