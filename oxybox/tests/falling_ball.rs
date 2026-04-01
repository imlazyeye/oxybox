use glam::vec2;
use oxybox::*;

#[test]
fn falling_ball() {
    let world = World::new(1.0 / 60.0);
    world.set_gravity(vec2(0.0, -10.0));

    let _ground = world.create_body(&BodyDefinition {
        build_shape: BuildShape::Rectangle {
            dimensions: vec2(100.0, 20.0),
        },
        position: Some(vec2(0.0, -10.0)),
        ..Default::default()
    });

    let ball = world.create_body(&BodyDefinition {
        build_shape: BuildShape::Circle {
            center: glam::Vec2::ZERO,
            radius: 5.0,
        },
        position: Some(vec2(0.0, 20.0)),
        restitution: Some(0.0),
        kind: BodyKind::Dynamic,
        ..Default::default()
    });

    for _ in 0..120 {
        world.step();
    }

    let position = ball.position();
    assert!((5.0 - position.y) < 1e-3, "ball at wrong position: {position:?}");
}
