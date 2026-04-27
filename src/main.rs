use std::f32::consts::PI;

use bevy::{asset::AssetMetaCheck, core_pipeline::tonemapping::Tonemapping, prelude::*};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use environment::EnvState;
use shader::ShaderMaterial;

mod environment;
mod paintings;
mod sdf_text;
mod shader;
mod touch;

fn main() {
    let mut app = App::new();

    app.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 0.8,
        affects_lightmapped_meshes: false,
    })
    .add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(MaterialPlugin::<ShaderMaterial>::default())
    .add_plugins(MaterialPlugin::<sdf_text::SdfTextMaterial>::default())
    .add_plugins(NoCameraPlayerPlugin)
    .add_plugins(touch::TouchCameraPlugin)
    .add_plugins(environment::EnvironmentPlugin)
    .add_plugins(paintings::PaintingsPlugin)
    .add_systems(Startup, spawn_camera)
    .add_systems(OnEnter(EnvState::Ready), on_load);

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Tonemapping::Reinhard,
        Projection::Perspective(PerspectiveProjection {
            fov: PI / 4. * 1.2,
            ..default()
        }),
        Transform::from_xyz(0., 1.7, 6.).looking_at(Vec3::new(0., 1.7, -12.), Vec3::Y),
        Visibility::default(),
        FlyCam,
    ));
}

fn on_load(mut commands: Commands) {
    // General fill light
    commands.spawn((
        PointLight {
            intensity: 1_500_000.0,
            range: 40.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0., 8.0, 0.),
    ));

    // Spot lights aimed at each painting
    let warm_white = Color::srgb(1.0, 0.98, 0.95);
    for &lx in &[-6.0_f32, -2.0, 2.0, 6.0] {
        commands.spawn((
            SpotLight {
                intensity: 3_000_000.0,
                range: 15.0,
                color: warm_white,
                outer_angle: PI / 6.0,
                inner_angle: PI / 8.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(lx, 7.0, -8.0).looking_at(Vec3::new(lx, 1.7, -12.0), Vec3::Y),
        ));
    }

    // Spot light aimed at text wall (right wall, far end)
    commands.spawn((
        SpotLight {
            intensity: 2_500_000.0,
            range: 15.0,
            color: warm_white,
            outer_angle: PI / 5.0,
            inner_angle: PI / 7.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.0, 6.0, 6.0).looking_at(Vec3::new(9.0, 1.8, 6.0), Vec3::Y),
    ));
}
