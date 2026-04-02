use glam::Vec2;
use oxybox::*;

#[test]
fn falling_ball() {
    let world = World::new(1.0 / 60.0);
    world.set_gravity(Vec2::new(0.0, -10.0));

    let ground = world.create_body(&BodyDefinition::new().position(Vec2::new(0.0, -10.0)));
    ground.attach_rectangle(Vec2::new(50.0, 10.0), Vec2::ZERO, 0.0, &ShapeDefinition::default());

    let ball = world.create_body(
        &BodyDefinition::new()
            .position(Vec2::new(0.0, 20.0))
            .kind(BodyKind::Dynamic),
    );
    ball.attach_circle(Vec2::ZERO, 5.0, &ShapeDefinition::new().restitution(0.0));

    for _ in 0..120 {
        world.step();
    }

    let position = ball.position();
    assert!((5.0 - position.y) < 1e-3, "ball at wrong position: {position:?}");
}
