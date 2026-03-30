fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=vendor/box2d/include/box2d/box2d.h");
    println!("cargo::rerun-if-changed=vendor/box2d/src");

    let mut ccbuild = cc::Build::new();
    ccbuild.include("vendor/box2d/include").warnings(false);

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
        "world.c",
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
        ccbuild.file(format!("vendor/box2d/src/{}", f));
    }
    ccbuild.compile("box2d");
    println!("cargo::rustc-link-lib=static=box2d");
}
