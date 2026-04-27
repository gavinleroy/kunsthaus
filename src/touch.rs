use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_flycam::FlyCam;

const LOOK_SENSITIVITY: f32 = 0.004;
const MOVE_SENSITIVITY: f32 = 0.01;
const PITCH_LIMIT: f32 = PI / 2.0 - 0.05;

pub struct TouchCameraPlugin;

impl Plugin for TouchCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchState>()
            .add_systems(Update, touch_camera);
    }
}

#[derive(Resource, Default)]
struct TouchState {
    prev_single: Option<Vec2>,
    prev_multi: Option<(Vec2, Vec2)>,
}

fn touch_camera(
    touches: Res<Touches>,
    mut state: ResMut<TouchState>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    let active: Vec<_> = touches.iter().collect();

    match active.len() {
        1 => {
            let pos = active[0].position();
            if let Some(prev) = state.prev_single {
                let delta = pos - prev;
                for mut t in &mut query {
                    let (mut yaw, mut pitch, _) = t.rotation.to_euler(EulerRot::YXZ);
                    yaw -= delta.x * LOOK_SENSITIVITY;
                    pitch -= delta.y * LOOK_SENSITIVITY;
                    pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
                    t.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
                }
            }
            state.prev_single = Some(pos);
            state.prev_multi = None;
        }
        2 => {
            let a = active[0].position();
            let b = active[1].position();
            if let Some((pa, pb)) = state.prev_multi {
                let prev_center = (pa + pb) * 0.5;
                let center = (a + b) * 0.5;
                let delta = center - prev_center;

                for mut t in &mut query {
                    let forward = t.forward().as_vec3() * Vec3::new(1.0, 0.0, 1.0);
                    let right = t.right().as_vec3() * Vec3::new(1.0, 0.0, 1.0);
                    t.translation -= forward.normalize_or_zero() * delta.y * MOVE_SENSITIVITY;
                    t.translation += right.normalize_or_zero() * delta.x * MOVE_SENSITIVITY;
                }
            }
            state.prev_multi = Some((a, b));
            state.prev_single = None;
        }
        _ => {
            state.prev_single = None;
            state.prev_multi = None;
        }
    }
}
