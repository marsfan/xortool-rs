use std::io::Read;

pub fn main() {
    todo!()
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
    s.bytes().collect()
}

fn from_file(s: &str) -> Vec<u8> {
    if s == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read(&mut buf).unwrap();
        return buf;
    }
    std::fs::read(s).unwrap()
}

fn arg_data(opt: &str, s: &str) -> Vec<u8> {
    match opt {
        "-s" => from_str(s),
        "-r" => s.bytes().collect(),
        //FIXME: There has to be a way to make this a bit nicer looking
        "-h" => s
            .replace(" ", "")
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|c| c.iter().collect::<String>())
            .map(|c| u8::from_str_radix(&c, 16).unwrap())
            .collect(),
        "-f" => from_file(s),
        _ => panic!("Unknown Option {opt}"),
    }
}
