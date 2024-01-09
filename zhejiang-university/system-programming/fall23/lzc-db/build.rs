use std::fs;
use std::path::Path;

fn main() {
    let source_path = Path::new("cfg.bin");
    let target_path = Path::new(&std::env::var("OUT_DIR").unwrap()).join("cfg.bin");
    if let Err(err) = fs::copy(source_path, target_path) {
        eprintln!("Error copying cfg.bin: {}", err);
    }
}
