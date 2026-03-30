fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/box2d/include/box2d/box2d.h");
    println!("cargo:rerun-if-changed=vendor/box2d/src");

    let box2d = camino::Utf8Path::new("vendor/box2d");

    // Build the vendored C library (Box2D 3.x is C)
    let mut ccbuild = cc::Build::new();
    ccbuild.include(box2d.join("include")).warnings(false);

    let src = box2d.join("src");
    for f in [
        "aabb.c",
        "arena_allocator.c",
        "array.c",
        "bitset.c",
        "body.c",
        "broad_phase.c",
        "constraint_graph.c",
        "contact.c",
        "contact_solver.c",
        "core.c",
        "distance.c",
        "distance_joint.c",
        "dynamic_tree.c",
        "geometry.c",
        "hull.c",
        "id_pool.c",
        "island.c",
        "joint.c",
        "manifold.c",
        "math_functions.c",
        "motor_joint.c",
        "mover.c",
        "physics_world.c",
        "prismatic_joint.c",
        "revolute_joint.c",
        "sensor.c",
        "shape.c",
        "solver_set.c",
        "solver.c",
        "table.c",
        "timer.c",
        "types.c",
        "weld_joint.c",
        "wheel_joint.c",
    ] {
        ccbuild.file(src.join(f));
    }
    ccbuild.compile("box2d");
    println!("cargo:rustc-link-lib=static=box2d");

    // regenerate bindings when feature is enabled
    #[cfg(feature = "bindgen")]
    {
        use std::env;
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let bindings = bindgen::Builder::default()
            .header(box2d.join("include/box2d/box2d.h"))
            .clang_arg(format!("-I{}", box2d.join("include")))
            .allowlist_function("b2.*")
            .allowlist_type("b2.*")
            .allowlist_var("b2.*")
            .wrap_unsafe_ops(true)
            .generate_inline_functions(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("bindgen failed");
        bindings
            .write_to_file(out_dir.join("bindings_gen.rs"))
            .expect("write bindings_gen.rs failed");
        println!(
            "cargo::warning=Bindings generated at: {}/bindings_gen.rs",
            out_dir.display()
        );
    }
}
