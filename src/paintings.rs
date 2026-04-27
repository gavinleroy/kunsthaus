use bevy::prelude::*;

use crate::environment::{EnvState, ROOM_SCALE};
use crate::sdf_text::SdfTextMaterial;
use crate::typography::*;

const ROOM_HALF_X: f32 = 2.25 * ROOM_SCALE;
const ROOM_HALF_Z: f32 = 3.0 * ROOM_SCALE;
const BACK_WALL_Z: f32 = -ROOM_HALF_Z;
const RIGHT_WALL_X: f32 = ROOM_HALF_X;
const WALL_OFFSET: f32 = 0.05;
const PAINTING_WIDTH: f32 = 2.0;
const PAINTING_CENTER_Y: f32 = 1.7;
const FRAME_BAR: f32 = 0.08;
const FRAME_DEPTH: f32 = 0.06;
const MAT_WIDTH: f32 = 0.12;

const PAINTING_XS: [f32; 4] = [-6.0, -2.0, 2.0, 6.0];

const FONT_TITLE: &[u8] = include_bytes!(
    "../assets/fonts/suisse-desktop/Suisse font/desktop files/SuisseIntlCond-Semibold.otf"
);
const FONT_BODY: &[u8] = include_bytes!("../assets/fonts/the-w/desktop files/TheWNYC-Regular.otf");

const GALLERY_TITLE: &str = "kunst\nhaus";
const GALLERY_DESCRIPTION: &str = "\
I am not an artist\n\
\n\
Though, I do make art. It turns out that there aren't good spaces for the art of non-artists\n\
\n\
I am a programmer, so why not put my art online?\n\
\n\
Welcome to my art gallery. It's a work in progress, but I hope you find it as fun as I do";
const DESC_WRAP_PX: f32 = 4500.0;
const DESC_FONT_SIZE: f32 = 350.0;

struct PrintInfo {
    path: &'static str,
    aspect_hw: f32,
    title: &'static str,
    date: &'static str,
    description: &'static str,
}

const PRINTS: &[PrintInfo] = &[
    PrintInfo {
        path: "prints/partially-furnished.jpg",
        aspect_hw: 3508.0 / 2481.0,
        title: "Partially-Furnished Party",
        date: "2025",
        description: "",
    },
    PrintInfo {
        path: "prints/the-second-coming.jpg",
        aspect_hw: 3300.0 / 2550.0,
        title: "Winter Vacation",
        date: "2025",
        description: "",
    },
    PrintInfo {
        path: "prints/xicara-de-sol.jpg",
        aspect_hw: 7016.0 / 4961.0,
        title: "Xícara de Sol",
        date: "2024",
        description: "",
    },
    PrintInfo {
        path: "prints/yard-sale.jpg",
        aspect_hw: 3300.0 / 2550.0,
        title: "The Hidden Yard Sale",
        date: "2026",
        description: "Printed on a map of Providence from 1912",
    },
];

fn create_wall_text(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_mats: ResMut<Assets<SdfTextMaterial>>,
) {
    let vinyl = LinearRgba::new(0.157, 0.157, 0.157, 1.0);
    let wall_x = RIGHT_WALL_X - WALL_OFFSET - 0.001;
    let rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
    let top_y = 3.2;

    let title_font = Font::new(FONT_TITLE, 360.0, vinyl).line_height(0.60);
    let title_box = TextBox::new(SdfSize::TotalWidth(3.0))
        .align(TextAlign::Left)
        .spread(60.0);
    spawn_text(
        &title_font,
        GALLERY_TITLE,
        &title_box,
        Transform::from_xyz(wall_x, top_y, 4.0).with_rotation(rotation),
        &mut commands,
        &mut images,
        &mut meshes,
        &mut sdf_mats,
    );

    let body_font = Font::new(FONT_BODY, DESC_FONT_SIZE, vinyl);
    let desc_box = TextBox::new(SdfSize::TotalWidth(1.9))
        .width(DESC_WRAP_PX)
        .align(TextAlign::Left)
        .spread(30.0);
    spawn_text(
        &body_font,
        GALLERY_DESCRIPTION,
        &desc_box,
        Transform::from_xyz(wall_x, top_y - 0.25, 6.8).with_rotation(rotation),
        &mut commands,
        &mut images,
        &mut meshes,
        &mut sdf_mats,
    );
}

fn create_paintings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sdf_mats: ResMut<Assets<SdfTextMaterial>>,
) {
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.15, 0.08),
        perceptual_roughness: 0.6,
        metallic: 0.1,
        reflectance: 0.3,
        ..default()
    });
    let mat_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.93, 0.90),
        perceptual_roughness: 0.9,
        ..default()
    });

    let label_color = LinearRgba::new(0.196, 0.196, 0.196, 1.0);
    let label_font = Font::new(FONT_BODY, 80.0, label_color);
    let label_box = TextBox::new(SdfSize::HeightPerLine(0.1))
        .width(1200.0)
        .align(TextAlign::Left);

    let mat_z = BACK_WALL_Z + WALL_OFFSET;
    let painting_z = mat_z + 0.003;
    let frame_z = mat_z + FRAME_DEPTH / 2.0;
    let label_z = mat_z + 0.002;

    for (print, &cx) in PRINTS.iter().zip(PAINTING_XS.iter()) {
        let pw = PAINTING_WIDTH;
        let ph = pw * print.aspect_hw;
        let outer_w = pw + 2.0 * MAT_WIDTH + 2.0 * FRAME_BAR;
        let inner_w = pw + 2.0 * MAT_WIDTH;
        let inner_h = ph + 2.0 * MAT_WIDTH;

        let texture = asset_server.load(print.path);
        let painting_mat = materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            ..default()
        });

        let mat_mesh = meshes.add(Rectangle::new(inner_w, inner_h));
        commands.spawn((
            Mesh3d(mat_mesh),
            MeshMaterial3d(mat_mat.clone()),
            Transform::from_xyz(cx, PAINTING_CENTER_Y, mat_z),
            Visibility::default(),
        ));

        let painting_mesh = meshes.add(Rectangle::new(pw, ph));
        commands.spawn((
            Mesh3d(painting_mesh),
            MeshMaterial3d(painting_mat),
            Transform::from_xyz(cx, PAINTING_CENTER_Y, painting_z),
            Visibility::default(),
        ));

        let top_mesh = meshes.add(Cuboid::new(outer_w, FRAME_BAR, FRAME_DEPTH));
        let bot_mesh = meshes.add(Cuboid::new(outer_w, FRAME_BAR, FRAME_DEPTH));
        let left_mesh = meshes.add(Cuboid::new(FRAME_BAR, inner_h, FRAME_DEPTH));
        let right_mesh = meshes.add(Cuboid::new(FRAME_BAR, inner_h, FRAME_DEPTH));

        let half_h = inner_h / 2.0 + FRAME_BAR / 2.0;
        let half_w = inner_w / 2.0 + FRAME_BAR / 2.0;

        for (mesh, offset) in [
            (top_mesh, Vec3::new(0.0, half_h, 0.0)),
            (bot_mesh, Vec3::new(0.0, -half_h, 0.0)),
            (left_mesh, Vec3::new(-half_w, 0.0, 0.0)),
            (right_mesh, Vec3::new(half_w, 0.0, 0.0)),
        ] {
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(frame_mat.clone()),
                Transform::from_translation(Vec3::new(cx, PAINTING_CENTER_Y, frame_z) + offset),
                Visibility::default(),
            ));
        }

        let label_text = if print.description.is_empty() {
            format!("{}\n{}", print.title, print.date)
        } else {
            format!("{}\n{}\n\n{}", print.title, print.date, print.description)
        };
        let label_x = cx + inner_w / 2.0 + FRAME_BAR + 0.1;
        let frame_bottom = PAINTING_CENTER_Y - inner_h / 2.0 - FRAME_BAR;
        let frame_top = PAINTING_CENTER_Y + inner_h / 2.0 + FRAME_BAR;
        let label_y = frame_bottom + (frame_top - frame_bottom) * 0.6;
        spawn_text(
            &label_font,
            &label_text,
            &label_box,
            Transform::from_xyz(label_x, label_y, label_z),
            &mut commands,
            &mut images,
            &mut meshes,
            &mut sdf_mats,
        );
    }
}

pub struct PaintingsPlugin;
impl Plugin for PaintingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(EnvState::Ready),
            (create_paintings, create_wall_text),
        );
    }
}
