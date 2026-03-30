use glam::vec2;
use oxybox::*;

#[test]
fn falling_ball() {
    let world = World::new(1.0 / 60.0);
    world.set_gravity(vec2(0.0, -10.0));

    let _ground = BodyBuilder::rectangle(vec2(100.0, 20.0))
        .position(vec2(0.0, -10.0))
        .build(world.id());

    let ball = BodyBuilder::circle(5.0)
        .kind(BodyKind::Dynamic)
        .position(vec2(0.0, 20.0))
        .restitution(0.0)
        .build(world.id());

    for _ in 0..120 {
        world.step();
    }

    let position = ball.position();
    assert!((5.0 - position.y) < 1e-3, "ball at wrong position: {position:?}");
}
