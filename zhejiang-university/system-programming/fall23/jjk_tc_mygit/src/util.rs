use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fmt::Write;
use std::num::ParseIntError;
use std::path::Path;

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).expect("hex encoding failed");
    }
    s
}

pub fn generate_temp_name() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(6).map(char::from).collect()
}

pub fn relative_path_from(path: &Path, from: &Path) -> String {
    path.strip_prefix(from)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
