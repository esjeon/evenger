
use bindgen;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("bindgen.rs");

    if out_path.exists() {
        return;
    }

    bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
            .expect("can't generate binding")
        .write_to_file(out_path)
            .expect("can't write binding to file");
}
