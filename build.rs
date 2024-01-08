fn main() {
    use std::env;
    use std::path::PathBuf;
    println!("cargo:rerun-if-changed=src/ui/uifile.fl");
    let g = fl2rust::Generator::default();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    g.in_out(
        "src/ui/uifile.fl",
        out_path.join("uifile.rs").to_str().unwrap(),
    )
    .expect("Failed to generate rust from fl file!");
}
