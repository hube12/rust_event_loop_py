extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_name = env::var("CARGO_PKG_NAME").unwrap();
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(format!("target/{}.h",project_name));
}
