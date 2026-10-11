fn collect(dir: &std::path::Path, root: &std::path::Path, output: &mut String) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(&path, root, output);
        } else {
            let key = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            output.push_str(&format!("{key:?} => Some(include_bytes!({path:?})),\n"));
        }
    }
}
fn main() {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf();
    println!("cargo:rerun-if-changed=../assets");
    let mut output =
        String::from("pub fn asset(path: &str) -> Option<&'static [u8]> { match path {\n");
    collect(&root.join("assets"), &root, &mut output);
    output.push_str("_ => None } }");
    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("assets.rs"),
        output,
    )
    .unwrap();
    println!("cargo:rerun-if-changed=../app_icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../app_icon.ico")
            .set("ProductName", "Diner Dash")
            .set("FileDescription", "Diner Dash - Made by Nirusaki")
            .set("CompanyName", "Nirusaki")
            .set("LegalCopyright", "Made by Nirusaki")
            .set("Comments", "Made by Nirusaki");
        res.compile()
            .expect("compile Windows icon and version metadata");
    }
}
