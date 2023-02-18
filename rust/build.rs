extern crate bindgen;

fn main() {
    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/../";
    println!("cargo:rustc-link-search={}", &root_dir);
    println!("cargo:rustc-link-lib=condux");
    if let Ok(os) = std::env::var("CARGO_CFG_TARGET_OS") {
        match os.as_str() {
            "horizon" => {
                let dkp = std::env::var("DEVKITPRO").unwrap();
                println!("cargo:rustc-link-search={}/libctru/lib", dkp);
                println!("cargo:rustc-link-lib=citro2d");
                println!("cargo:rustc-link-lib=citro3d");
                println!("cargo:rustc-link-lib=ctru");
            }

            _ => {
                println!("cargo:rustc-link-lib=SDL2");
                println!("cargo:rustc-link-lib=GL");
            }
        }
    }
    // TODO we need to rerun for *all* files in the include dir
    for path in std::fs::read_dir(root_dir.clone() + "include").unwrap() {
        println!("cargo:rerun-if-changed={}", path.unwrap().path().display());
    }
    let builder = bindgen::Builder::default()
        .header(root_dir + "include/bindings.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
    let bindings = builder.generate().unwrap();
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).unwrap();
}
