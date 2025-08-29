use glam::vec2;
use oxybox::*;

#[test]
fn falling_ball() {
    let mut world = World::default();
    world.set_gravity(vec2(0.0, -10.0));

    let _ground = BodyBuilder::rectangle(vec2(100.0, 20.0))
        .position(vec2(0.0, -10.0))
        .build(&mut world);

    let ball = BodyBuilder::circle(5.0)
        .kind(BodyKind::Dynamic)
        .position(vec2(0.0, 20.0))
        .restitution(0.0)
        .build(&mut world);

    for _ in 0..120 {
        world.tick();
    }

    let position = world.read(ball).unwrap().position();
    assert!((5.0 - position.y) < 1e-3, "ball at wrong position: {position:?}");
}
