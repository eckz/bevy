//! This example illustrates the [`UiScale`] resource from `bevy_ui`.

use bevy::prelude::*;

const ANIMATION_TIME_SECS: f32 = 0.6;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (animate, set_animation).chain())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new(concat!(
            "Press 0: none.\n",
            "Press 1: translate(120%, -30%).\n",
            "Press 2: translate(150px, 20px) rotate(45deg).\n",
            "Press 3: scale(1.5) rotate(45.deg) translate(150px, 0).\n",
            "Press 4: translate(-100px, 0) rotateY(67.5deg).\n",
        )),
        TextFont::from_font_size(14.0),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(80.),
                height: Val::Percent(70.),
                position_type: PositionType::Absolute,
                left: Val::Percent(10.),
                top: Val::Percent(15.),
                ..default()
            },
           // BackgroundColor(ANTIQUE_WHITE.into()),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(150.0),
                        height: Val::Px(150.0),
                        top: Val::Percent(50.0),
                        left: Val::Percent(50.0),
                        ..default()
                    },
                    Outline {
                        width: Val::Px(2.0),
                        offset: Val::ZERO,
                        color: Color::BLACK,
                    },
                    NodeTransform::from_translate3d(Val::Percent(-50.), Val::Percent(-50.), Val::Px(400.0)),
                ))
                .with_child((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode::new(asset_server.load("branding/icon.png")),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderRadius::all(Val::Px(20.0)),
                    BoxShadow::default(),
                    NodeTransform::default(),
                    TransformAnimatable,
                ));
        });
}

fn set_animation(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<
        (Entity, &NodeTransform),
        (With<TransformAnimatable>, Without<TransformAnimation>),
    >,
) {
    let Ok((entity, from)) = query.get_single() else {
        return;
    };

    let to = if input.just_pressed(KeyCode::Digit0) {
        NodeTransform::default()
    } else if input.just_pressed(KeyCode::Digit1) {
        NodeTransform::from_translate(Val::Percent(120.), Val::Percent(-30.))
    } else if input.just_pressed(KeyCode::Digit2) {
        NodeTransform::from_translate(Val::Px(150.), Val::Px(20.)).with_rotate(f32::to_radians(45.))
    } else if input.just_pressed(KeyCode::Digit3) {
        NodeTransform::from_scale(Vec2::splat(1.5))
            .with_rotate(f32::to_radians(45.))
            .with_translate(Val::Px(150.), Val::ZERO)
    } else if input.just_pressed(KeyCode::Digit4) {
        NodeTransform::from_translate(Val::Px(-100.), Val::ZERO)
            .with_rotate3d(Dir3::Y, f32::to_radians(67.5))
    } else {
        return;
    };

    commands
        .entity(entity)
        .insert(TransformAnimation::new(from.clone(), to));
}

fn animate(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut NodeTransform, &mut TransformAnimation)>,
) {
    for (entity, mut node_transform, mut animation) in &mut query {
        animation.animation_timer.tick(time.delta());
        *node_transform = animation.current_value();

        if animation.animation_timer.finished() {
            commands.entity(entity).remove::<TransformAnimation>();
        }
    }
}

#[derive(Component)]
struct TransformAnimatable;

#[derive(Component)]
struct TransformAnimation {
    from: NodeTransform,
    to: NodeTransform,
    from_transform: Transform,
    to_transform: Transform,
    ease: CubicSegment<Vec2>,
    animation_timer: Timer,
}

impl TransformAnimation {
    fn new(from: NodeTransform, to: NodeTransform) -> Self {
        let parent_size = Vec2::new(150.0, 150.0);
        let from_transform = from.resolve(parent_size, Vec2::NAN, 1.0).unwrap();
        let to_transform = to.resolve(parent_size, Vec2::NAN, 1.0).unwrap();
        Self {
            from,
            to,
            from_transform,
            to_transform,
            ease: CubicSegment::new_bezier((0.25, 0.1), (0.25, 1.)),
            animation_timer: Timer::from_seconds(
                ANIMATION_TIME_SECS,
                TimerMode::Once,
            ),
        }
    }

    fn current_value(&self) -> NodeTransform {
        let t = self.animation_timer.elapsed_secs() / ANIMATION_TIME_SECS;
        if t <= 0.0 {
            return self.from.clone();
        } else if t >= 1.0 {
            return self.to.clone();
        }
        let eased_t = self.ease.ease(t);

        let interpolated_transform =
            Animatable::interpolate(&self.from_transform, &self.to_transform, eased_t);

        let Vec2 { x, y } = interpolated_transform.translation.truncate();
        let (axis, angle) = interpolated_transform.rotation.to_axis_angle();
        let scale = interpolated_transform.scale;

        NodeTransform::from_translate(Val::Px(x), Val::Px(y))
            .with_rotate3d(Dir3::new(axis).unwrap(), angle)
            .with_scale3d(scale)
    }
}
