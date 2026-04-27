use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::environment::{EnvState, ROOM_SCALE};
use crate::sdf_text::SdfTextMaterial;

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
Though, I do make art. It turns out that there aren’t good spaces for the art of non-artists\n\
\n\
I am a programmer, so why not put my art online?\n\
\n\
Welcome to my art gallery. It’s a work in progress, but I hope you find it as fun as I do";
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

// --- Text shaping + rasterization ---

fn shape_line(face: &rustybuzz::Face, text: &str) -> rustybuzz::GlyphBuffer {
    let mut buf = rustybuzz::UnicodeBuffer::new();
    buf.push_str(text);
    rustybuzz::shape(face, &[], buf)
}

fn wrap_text(font_data: &[u8], text: &str, font_size: f32, max_width_px: f32) -> String {
    use hyphenation::{Hyphenator, Language, Load, Standard};

    let face = rustybuzz::Face::from_slice(font_data, 0).expect("invalid font");
    let px_per_unit = font_size / face.units_per_em() as f32;
    let dict = Standard::from_embedded(Language::EnglishUS).expect("hyphenation dict");

    let measure = |s: &str| -> f32 {
        let shaped = shape_line(&face, s);
        shaped
            .glyph_positions()
            .iter()
            .map(|p| p.x_advance as f32 * px_per_unit)
            .sum()
    };

    let mut result = String::new();
    for (pi, paragraph) in text.split('\n').enumerate() {
        if pi > 0 {
            result.push('\n');
        }
        if paragraph.is_empty() {
            continue;
        }
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let mut line = String::new();
        for word in words {
            let test = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if measure(&test) <= max_width_px || line.is_empty() {
                line = test;
                continue;
            }
            // Word doesn't fit — try hyphenation at each break point (largest first)
            let hyphenated = dict.hyphenate(word);
            let mut placed = false;
            for &bi in hyphenated.breaks.iter().rev() {
                let candidate = format!("{line} {}-", &word[..bi]);
                if measure(&candidate) <= max_width_px {
                    result.push_str(&candidate);
                    result.push('\n');
                    line = word[bi..].to_string();
                    placed = true;
                    break;
                }
            }
            if !placed {
                result.push_str(&line);
                result.push('\n');
                line = word.to_string();
            }
        }
        if !line.is_empty() {
            result.push_str(&line);
        }
    }
    result
}

fn rasterize_alpha(
    font_data: &[u8],
    text: &str,
    font_size: f32,
    padding: u32,
    line_spacing: f32,
) -> (Vec<u8>, u32, u32) {
    let rb_face = rustybuzz::Face::from_slice(font_data, 0).expect("invalid font for shaping");
    let upem = rb_face.units_per_em() as f32;
    let px_per_unit = font_size / upem;

    let ab_font = FontRef::try_from_slice(font_data).expect("invalid font");
    let scale = PxScale::from(font_size);
    let scaled = ab_font.as_scaled(scale);

    let lines: Vec<&str> = text.lines().collect();
    let effective_line_height = scaled.height() * line_spacing;
    let line_height = effective_line_height.ceil() as u32;

    // Shape each line with rustybuzz (handles GPOS kerning)
    let shaped: Vec<_> = lines.iter().map(|l| shape_line(&rb_face, l)).collect();

    let img_w = shaped
        .iter()
        .map(|out| {
            let w: i32 = out.glyph_positions().iter().map(|p| p.x_advance).sum();
            (w as f32 * px_per_unit).ceil() as u32
        })
        .max()
        .unwrap_or(1)
        + padding * 2;
    let img_h = line_height * lines.len() as u32 + padding * 2;

    let mut alpha = vec![0u8; (img_w * img_h) as usize];

    for (li, out) in shaped.iter().enumerate() {
        let y_base = padding as f32 + li as f32 * effective_line_height;
        let mut cursor_x = padding as f32;

        for (info, pos) in out.glyph_infos().iter().zip(out.glyph_positions()) {
            let gid = ab_glyph::GlyphId(info.glyph_id as u16);
            let x = cursor_x + pos.x_offset as f32 * px_per_unit;
            let y = y_base + scaled.ascent() - pos.y_offset as f32 * px_per_unit;

            let glyph = gid.with_scale_and_position(scale, ab_glyph::point(x, y));
            if let Some(outlined) = scaled.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|px, py, cov| {
                    let ix = bounds.min.x as i32 + px as i32;
                    let iy = bounds.min.y as i32 + py as i32;
                    if ix >= 0 && iy >= 0 && (ix as u32) < img_w && (iy as u32) < img_h {
                        let idx = (iy as u32 * img_w + ix as u32) as usize;
                        alpha[idx] = alpha[idx].max((cov * 255.0) as u8);
                    }
                });
            }

            cursor_x += pos.x_advance as f32 * px_per_unit;
        }
    }

    (alpha, img_w, img_h)
}

// --- SDF generation ---

fn chamfer_distance(inside: &[bool], w: u32, h: u32) -> Vec<f32> {
    const BIG: f32 = 1e10;
    const D1: f32 = 1.0;
    const D2: f32 = 1.414;
    let n = (w * h) as usize;
    let mut dist = vec![BIG; n];
    let idx = |x: i32, y: i32| -> usize { (y as u32 * w + x as u32) as usize };

    // Seed: edge pixels get distance 0
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let i = idx(x, y);
            let v = inside[i];
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 && inside[idx(nx, ny)] != v
                {
                    dist[i] = 0.0;
                    break;
                }
            }
        }
    }

    // Forward pass (top-left → bottom-right)
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let i = idx(x, y);
            let neighbors: [(i32, i32, f32); 4] = [
                (x - 1, y, D1),
                (x, y - 1, D1),
                (x - 1, y - 1, D2),
                (x + 1, y - 1, D2),
            ];
            for (nx, ny, cost) in neighbors {
                if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                    let nd = dist[idx(nx, ny)] + cost;
                    if nd < dist[i] {
                        dist[i] = nd;
                    }
                }
            }
        }
    }

    // Backward pass (bottom-right → top-left)
    for y in (0..h as i32).rev() {
        for x in (0..w as i32).rev() {
            let i = idx(x, y);
            let neighbors: [(i32, i32, f32); 4] = [
                (x + 1, y, D1),
                (x, y + 1, D1),
                (x + 1, y + 1, D2),
                (x - 1, y + 1, D2),
            ];
            for (nx, ny, cost) in neighbors {
                if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                    let nd = dist[idx(nx, ny)] + cost;
                    if nd < dist[i] {
                        dist[i] = nd;
                    }
                }
            }
        }
    }

    dist
}

struct SdfImage {
    image: Image,
    pad_frac: f32,
}

fn generate_sdf(
    font_data: &[u8],
    text: &str,
    font_size: f32,
    spread: f32,
    downsample: u32,
    line_spacing: f32,
) -> SdfImage {
    let pad = (spread as u32) + 8;
    let (alpha, w, h) = rasterize_alpha(font_data, text, font_size, pad, line_spacing);

    let inside: Vec<bool> = alpha.iter().map(|&a| a > 127).collect();
    let dist = chamfer_distance(&inside, w, h);

    // Build signed distance: positive inside, negative outside, normalized
    let mut sdf = vec![0.0f32; (w * h) as usize];
    for i in 0..(w * h) as usize {
        let signed = if inside[i] { dist[i] } else { -dist[i] };
        sdf[i] = (signed / spread * 0.5 + 0.5).clamp(0.0, 1.0);
    }

    // Downsample
    let out_w = w / downsample;
    let out_h = h / downsample;
    let mut pixels = vec![0u8; (out_w * out_h * 4) as usize];

    for oy in 0..out_h {
        for ox in 0..out_w {
            let mut sum = 0.0f32;
            let mut count = 0.0f32;
            for dy in 0..downsample {
                for dx in 0..downsample {
                    let hx = ox * downsample + dx;
                    let hy = oy * downsample + dy;
                    if hx < w && hy < h {
                        sum += sdf[(hy * w + hx) as usize];
                        count += 1.0;
                    }
                }
            }
            let val = (sum / count * 255.0) as u8;
            let idx = (oy * out_w + ox) as usize * 4;
            pixels[idx] = 255;
            pixels[idx + 1] = 255;
            pixels[idx + 2] = 255;
            pixels[idx + 3] = val;
        }
    }

    let pad_frac = (pad / downsample) as f32 / out_h as f32;
    SdfImage {
        image: Image::new(
            Extent3d {
                width: out_w,
                height: out_h,
                ..default()
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8Unorm,
            default(),
        ),
        pad_frac,
    }
}

// --- Spawning helpers ---

#[allow(dead_code)]
enum VAlign {
    Center,
    Top,
}

fn spawn_sdf_quad(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    sdf_mats: &mut Assets<SdfTextMaterial>,
    sdf: SdfImage,
    world_w: f32,
    mut transform: Transform,
    color: LinearRgba,
    valign: VAlign,
) {
    let w = sdf.image.width() as f32;
    let h = sdf.image.height() as f32;
    let world_h = world_w * (h / w);
    if matches!(valign, VAlign::Top) {
        // Align visual text top (not quad top) to the given y
        transform.translation.y -= world_h * (0.5 - sdf.pad_frac);
    }
    let handle = images.add(sdf.image);
    let mat = sdf_mats.add(SdfTextMaterial {
        sdf_texture: handle,
        text_color: color,
    });
    let mesh = meshes.add(Rectangle::new(world_w, world_h));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        transform,
        Visibility::default(),
    ));
}

// --- Systems ---

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

    // Left column: title "kunst / haus"
    let title_sdf = generate_sdf(FONT_TITLE, GALLERY_TITLE, 360.0, 60.0, 2, 0.60);
    spawn_sdf_quad(
        &mut commands,
        &mut images,
        &mut meshes,
        &mut sdf_mats,
        title_sdf,
        3.,
        Transform::from_xyz(wall_x, top_y, 4.5).with_rotation(rotation),
        vinyl,
        VAlign::Top,
    );

    // Right column: word-wrapped description, manually lowered to match title baseline
    let wrapped = wrap_text(FONT_BODY, GALLERY_DESCRIPTION, DESC_FONT_SIZE, DESC_WRAP_PX);
    let desc_sdf = generate_sdf(FONT_BODY, &wrapped, DESC_FONT_SIZE, 30.0, 2, 1.0);
    spawn_sdf_quad(
        &mut commands,
        &mut images,
        &mut meshes,
        &mut sdf_mats,
        desc_sdf,
        1.9,
        Transform::from_xyz(wall_x, top_y - 0.25, 6.8).with_rotation(rotation),
        vinyl,
        VAlign::Top,
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

        // Artwork label — SDF rendered
        let label_font_size = 220.0;
        let label_wrap_px = 1200.0;
        let label_text = if print.description.is_empty() {
            format!("{}\n{}", print.title, print.date)
        } else {
            let wrapped_desc =
                wrap_text(FONT_BODY, print.description, label_font_size, label_wrap_px);
            format!("{}\n{}\n\n{}", print.title, print.date, wrapped_desc)
        };
        let label_sdf = generate_sdf(FONT_BODY, &label_text, label_font_size, 32.0, 2, 1.0);
        let label_x = cx + inner_w / 2.0 + FRAME_BAR + 0.55;
        let frame_bottom = PAINTING_CENTER_Y - inner_h / 2.0 - FRAME_BAR;
        let frame_top = PAINTING_CENTER_Y + inner_h / 2.0 + FRAME_BAR;
        let label_y = frame_bottom + (frame_top - frame_bottom) * 0.6;
        spawn_sdf_quad(
            &mut commands,
            &mut images,
            &mut meshes,
            &mut sdf_mats,
            label_sdf,
            1.0,
            Transform::from_xyz(label_x, label_y, label_z),
            label_color,
            VAlign::Top,
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
