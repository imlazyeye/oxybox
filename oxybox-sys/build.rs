fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=vendor/box2d/include/box2d/box2d.h");
    println!("cargo::rerun-if-changed=vendor/box2d/src");

    let mut ccbuild = cc::Build::new();
    ccbuild.include("vendor/box2d/include").warnings(false);

    for entry in std::fs::read_dir("vendor/box2d/src").unwrap() {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "c") {
            ccbuild.file(path);
        }
    }

    ccbuild.compile("box2d");
    println!("cargo::rustc-link-lib=static=box2d");
}
