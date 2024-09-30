fn main() -> std::io::Result<()> {
    // build the third-party C library
    cc::Build::new()
        .file("third-party/src/vector.c")
        .include("third-party/include")
        .compile("vector");

    // generate bindings for the third-party C library
    let bindings = bindgen::Builder::default()
        .header("third-party/include/vector.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("could not generate bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("could not write bindings");

    Ok(())
}
