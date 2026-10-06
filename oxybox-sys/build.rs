fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=vendor/box2d/include/box2d/box2d.h");
    println!("cargo::rerun-if-changed=vendor/box2d/src");

    let mut ccbuild = cc::Build::new();
    ccbuild.include("vendor/box2d/include").warnings(false);

    let flag = if ccbuild.get_compiler().is_like_msvc() {
        "/std:c17"
    } else {
        "-std=gnu17"
    };
    ccbuild.flag(flag);

    // tie B2_ASSERT to debug_assertions, like `debug_assert!`
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
        ccbuild.define("NDEBUG", None);
    }

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
