use bevy::{prelude::*, scene::InstanceId};
use bevy_flycam::FlyCam;

pub const ROOM_SCALE: f32 = 4.0;

const ROOM_HALF_X: f32 = 2.25 * ROOM_SCALE;
const ROOM_HALF_Z: f32 = 3.0 * ROOM_SCALE;
const ROOM_HEIGHT: f32 = 2.44 * ROOM_SCALE;
const ROOM_MARGIN: f32 = 0.3;
const EYE_HEIGHT: f32 = 1.7;

#[derive(Resource, Default)]
struct Environment {
    instance: Option<InstanceId>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum EnvState {
    #[default]
    Loading,
    Ready,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut environment: ResMut<Environment>,
) {
    let handle = asset_server.load("models/shaderHall/shaderHall.gltf#Scene0");
    let entity = commands
        .spawn((
            Transform::from_scale(Vec3::splat(ROOM_SCALE)),
            Visibility::default(),
        ))
        .id();
    environment.instance = Some(scene_spawner.spawn_as_child(handle, entity));
}

fn clamp_player(mut query: Query<&mut Transform, With<FlyCam>>) {
    for mut t in &mut query {
        t.translation.x = t
            .translation
            .x
            .clamp(-ROOM_HALF_X + ROOM_MARGIN, ROOM_HALF_X - ROOM_MARGIN);
        t.translation.z = t
            .translation
            .z
            .clamp(-ROOM_HALF_Z + ROOM_MARGIN, ROOM_HALF_Z - ROOM_MARGIN);
        t.translation.y = EYE_HEIGHT;
    }
}

fn wait_for_load(
    scene_spawner: Res<SceneSpawner>,
    environment: Res<Environment>,
    mut next_state: ResMut<NextState<EnvState>>,
) {
    if let Some(id) = environment.instance {
        if scene_spawner.instance_is_ready(id) {
            next_state.set(EnvState::Ready);
        }
    }
}

fn fix_materials(mut materials: ResMut<Assets<StandardMaterial>>) {
    for (_, mat) in materials.iter_mut() {
        mat.double_sided = true;
        mat.cull_mode = None;
        mat.base_color = Color::srgb(0.95, 0.95, 0.95);
        mat.perceptual_roughness = 0.9;
        mat.metallic = 0.0;
    }
}

fn spawn_furniture(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hx = ROOM_HALF_X;
    let hz = ROOM_HALF_Z;
    let rh = ROOM_HEIGHT;

    // --- Materials ---
    let trim_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.92, 0.92),
        perceptual_roughness: 0.8,
        ..default()
    });
    let wood_floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.35, 0.18),
        perceptual_roughness: 0.4,
        metallic: 0.05,
        ..default()
    });
    let bench_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.30, 0.15),
        perceptual_roughness: 0.7,
        ..default()
    });
    let rug_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.08, 0.08),
        perceptual_roughness: 0.95,
        ..default()
    });
    let light_housing_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        perceptual_roughness: 0.9,
        metallic: 0.3,
        ..default()
    });

    // --- Floor overlay (wood) ---
    let floor = meshes.add(Cuboid::new(hx * 2.0 - 0.1, 0.01, hz * 2.0 - 0.1));
    commands.spawn((
        Mesh3d(floor),
        MeshMaterial3d(wood_floor_mat),
        Transform::from_xyz(0.0, 0.005, 0.0),
        Visibility::default(),
    ));

    // --- Baseboard molding ---
    let base_h = 0.15;
    let base_d = 0.03;
    let baseboards: [(Cuboid, Vec3); 4] = [
        (
            Cuboid::new(hx * 2.0, base_h, base_d),
            Vec3::new(0.0, base_h / 2.0, -hz + base_d / 2.0),
        ),
        (
            Cuboid::new(hx * 2.0, base_h, base_d),
            Vec3::new(0.0, base_h / 2.0, hz - base_d / 2.0),
        ),
        (
            Cuboid::new(base_d, base_h, hz * 2.0),
            Vec3::new(-hx + base_d / 2.0, base_h / 2.0, 0.0),
        ),
        (
            Cuboid::new(base_d, base_h, hz * 2.0),
            Vec3::new(hx - base_d / 2.0, base_h / 2.0, 0.0),
        ),
    ];
    for (cuboid, pos) in baseboards {
        let mesh = meshes.add(cuboid);
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(trim_mat.clone()),
            Transform::from_translation(pos),
            Visibility::default(),
        ));
    }

    // --- Crown molding ---
    let crown_h = 0.12;
    let crown_d = 0.04;
    let crown_y = rh - crown_h / 2.0;
    let crowns: [(Cuboid, Vec3); 4] = [
        (
            Cuboid::new(hx * 2.0, crown_h, crown_d),
            Vec3::new(0.0, crown_y, -hz + crown_d / 2.0),
        ),
        (
            Cuboid::new(hx * 2.0, crown_h, crown_d),
            Vec3::new(0.0, crown_y, hz - crown_d / 2.0),
        ),
        (
            Cuboid::new(crown_d, crown_h, hz * 2.0),
            Vec3::new(-hx + crown_d / 2.0, crown_y, 0.0),
        ),
        (
            Cuboid::new(crown_d, crown_h, hz * 2.0),
            Vec3::new(hx - crown_d / 2.0, crown_y, 0.0),
        ),
    ];
    for (cuboid, pos) in crowns {
        let mesh = meshes.add(cuboid);
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(trim_mat.clone()),
            Transform::from_translation(pos),
            Visibility::default(),
        ));
    }

    // --- Gallery bench ---
    let seat = meshes.add(Cuboid::new(2.4, 0.05, 0.5));
    commands.spawn((
        Mesh3d(seat),
        MeshMaterial3d(bench_mat.clone()),
        Transform::from_xyz(0.0, 0.45, 2.0),
        Visibility::default(),
    ));

    let leg = Cuboid::new(0.06, 0.425, 0.06);
    let leg_y = 0.425 / 2.0;
    let leg_positions = [
        Vec3::new(-1.10, leg_y, 1.81),
        Vec3::new(1.10, leg_y, 1.81),
        Vec3::new(-1.10, leg_y, 2.19),
        Vec3::new(1.10, leg_y, 2.19),
    ];
    for pos in leg_positions {
        let mesh = meshes.add(leg);
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(bench_mat.clone()),
            Transform::from_translation(pos),
            Visibility::default(),
        ));
    }

    let crossbar = meshes.add(Cuboid::new(2.0, 0.04, 0.04));
    commands.spawn((
        Mesh3d(crossbar),
        MeshMaterial3d(bench_mat.clone()),
        Transform::from_xyz(0.0, 0.15, 2.0),
        Visibility::default(),
    ));

    // --- Rug ---
    let rug = meshes.add(Cuboid::new(4.0, 0.02, 2.5));
    commands.spawn((
        Mesh3d(rug),
        MeshMaterial3d(rug_mat),
        Transform::from_xyz(0.0, 0.01, 2.0),
        Visibility::default(),
    ));

    // --- Track light housings ---
    let housing = Cuboid::new(0.15, 0.10, 0.25);
    for &lx in &[-6.0_f32, -2.0, 2.0, 6.0] {
        let mesh = meshes.add(housing);
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(light_housing_mat.clone()),
            Transform::from_xyz(lx, 7.0, -8.0),
            Visibility::default(),
        ));
    }

    // Text wall spotlight housing
    {
        let mesh = meshes.add(Cuboid::new(0.25, 0.10, 0.15));
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(light_housing_mat.clone()),
            Transform::from_xyz(5.0, 6.0, 6.0),
            Visibility::default(),
        ));
    }
}

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EnvState>()
            .init_resource::<Environment>()
            .add_systems(Update, wait_for_load.run_if(in_state(EnvState::Loading)))
            .add_systems(OnEnter(EnvState::Ready), (fix_materials, spawn_furniture))
            .add_systems(Update, clamp_player.run_if(in_state(EnvState::Ready)))
            .add_systems(Startup, setup);
    }
}
