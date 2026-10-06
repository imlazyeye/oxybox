fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=vendor/box2d/include");
    println!("cargo::rerun-if-changed=vendor/box2d/src");

    let mut ccbuild = cc::Build::new();
    ccbuild.include("vendor/box2d/include").warnings(false);

    let msvc = ccbuild.get_compiler().is_like_msvc();
    ccbuild.flag(if msvc { "/std:c17" } else { "-std=gnu17" });

    // follow the Rust target's features, as Box2D's CMake options would
    let target_features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let has_feature = |feature: &str| target_features.split(',').any(|f| f == feature);
    if has_feature("avx2") {
        ccbuild.define("BOX2D_AVX2", None);
        ccbuild.flag(if msvc { "/arch:AVX2" } else { "-mavx2" });
    }
    if std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "emscripten") {
        if has_feature("simd128") {
            ccbuild.flag("-msimd128").flag("-msse2");
        } else {
            ccbuild.define("BOX2D_DISABLE_SIMD", None);
        }
    }

    // matches Box2D's CMake; FMA contraction breaks cross-platform determinism
    ccbuild.flag_if_supported("-ffp-contract=off");

    // tie B2_ASSERT to debug_assertions, like `debug_assert!`
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
        ccbuild.define("NDEBUG", None);
    }

    let mut sources: Vec<_> = std::fs::read_dir("vendor/box2d/src")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "c"))
        .collect();
    sources.sort();
    ccbuild.files(sources);

    ccbuild.compile("box2d");
}
