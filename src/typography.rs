use ab_glyph::{Font as AbFont, FontRef, PxScale, ScaleFont};
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::sdf_text::SdfTextMaterial;

// --- Public types ---

#[derive(Clone, Copy)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
pub enum VAlign {
    Center,
    Top,
}

#[derive(Clone, Copy)]
pub enum SdfSize {
    TotalWidth(f32),
    HeightPerLine(f32),
}

pub struct Font<'a> {
    data: &'a [u8],
    rb_face: rustybuzz::Face<'a>,
    ab_font: FontRef<'a>,
    pub size: f32,
    pub color: LinearRgba,
    pub line_height: f32,
    pub tracking: f32,
    pub features: Vec<rustybuzz::Feature>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8], size: f32, color: LinearRgba) -> Self {
        let rb_face = rustybuzz::Face::from_slice(data, 0).expect("invalid font");
        let ab_font = FontRef::try_from_slice(data).expect("invalid font");
        Font {
            data,
            rb_face,
            ab_font,
            size,
            color,
            line_height: 1.0,
            tracking: 0.0,
            features: Vec::new(),
        }
    }

    pub fn line_height(mut self, lh: f32) -> Self {
        self.line_height = lh;
        self
    }

    pub fn tracking(mut self, t: f32) -> Self {
        self.tracking = t;
        self
    }

    pub fn features(mut self, f: Vec<rustybuzz::Feature>) -> Self {
        self.features = f;
        self
    }

    fn px_per_unit(&self) -> f32 {
        self.size / self.rb_face.units_per_em() as f32
    }

    fn shape_line(&self, text: &str) -> rustybuzz::GlyphBuffer {
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        rustybuzz::shape(&self.rb_face, &self.features, buf)
    }

    fn measure_line(&self, text: &str) -> f32 {
        let shaped = self.shape_line(text);
        let ppu = self.px_per_unit();
        shaped
            .glyph_positions()
            .iter()
            .map(|p| p.x_advance as f32 * ppu + self.tracking * ppu)
            .sum()
    }

    pub fn wrap(&self, text: &str, text_box: &TextBox) -> String {
        let max_width_px = match text_box.width {
            Some(w) => w,
            None => return text.to_string(),
        };

        use hyphenation::{Hyphenator, Language, Load, Standard};
        let dict = if text_box.hyphenate {
            Some(Standard::from_embedded(Language::EnglishUS).expect("hyphenation dict"))
        } else {
            None
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
                if self.measure_line(&test) <= max_width_px || line.is_empty() {
                    line = test;
                    continue;
                }
                let mut placed = false;
                if let Some(dict) = &dict {
                    let hyphenated = dict.hyphenate(word);
                    for &bi in hyphenated.breaks.iter().rev() {
                        let candidate = format!("{line} {}-", &word[..bi]);
                        if self.measure_line(&candidate) <= max_width_px {
                            result.push_str(&candidate);
                            result.push('\n');
                            line = word[bi..].to_string();
                            placed = true;
                            break;
                        }
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
        &self,
        text: &str,
        padding: u32,
        align: TextAlign,
    ) -> (Vec<u8>, u32, u32) {
        let ppu = self.px_per_unit();
        let scale = PxScale::from(self.size);
        let scaled = self.ab_font.as_scaled(scale);

        let lines: Vec<&str> = text.lines().collect();
        let effective_line_height = scaled.height() * self.line_height;
        let line_height = effective_line_height.ceil() as u32;

        let shaped: Vec<_> = lines.iter().map(|l| self.shape_line(l)).collect();

        let line_widths: Vec<f32> = shaped
            .iter()
            .map(|out| {
                out.glyph_positions()
                    .iter()
                    .map(|p| p.x_advance as f32 * ppu + self.tracking * ppu)
                    .sum()
            })
            .collect();

        let max_line_width = line_widths.iter().copied().fold(0.0f32, f32::max);
        let img_w = max_line_width.ceil() as u32 + padding * 2;
        let img_h = line_height * lines.len() as u32 + padding * 2;

        let mut alpha = vec![0u8; (img_w * img_h) as usize];

        for (li, out) in shaped.iter().enumerate() {
            let y_base = padding as f32 + li as f32 * effective_line_height;
            let align_offset = match align {
                TextAlign::Left => 0.0,
                TextAlign::Center => (max_line_width - line_widths[li]) / 2.0,
                TextAlign::Right => max_line_width - line_widths[li],
            };
            let mut cursor_x = padding as f32 + align_offset;

            for (info, pos) in out.glyph_infos().iter().zip(out.glyph_positions()) {
                let gid = ab_glyph::GlyphId(info.glyph_id as u16);
                let x = cursor_x + pos.x_offset as f32 * ppu;
                let y = y_base + scaled.ascent() - pos.y_offset as f32 * ppu;

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

                cursor_x += pos.x_advance as f32 * ppu + self.tracking * ppu;
            }
        }

        (alpha, img_w, img_h)
    }

    fn generate_sdf(&self, text: &str, text_box: &TextBox) -> SdfImage {
        let num_lines = text.lines().count() as u32;
        let pad = (text_box.spread as u32) + 8;
        let (alpha, w, h) = self.rasterize_alpha(text, pad, text_box.align);

        let inside: Vec<bool> = alpha.iter().map(|&a| a > 127).collect();
        let dist = chamfer_distance(&inside, w, h);

        let mut sdf = vec![0.0f32; (w * h) as usize];
        for i in 0..(w * h) as usize {
            let signed = if inside[i] { dist[i] } else { -dist[i] };
            sdf[i] = (signed / text_box.spread * 0.5 + 0.5).clamp(0.0, 1.0);
        }

        let ds = text_box.downsample;
        let out_w = w / ds;
        let out_h = h / ds;
        let mut pixels = vec![0u8; (out_w * out_h * 4) as usize];

        for oy in 0..out_h {
            for ox in 0..out_w {
                let mut sum = 0.0f32;
                let mut count = 0.0f32;
                for dy in 0..ds {
                    for dx in 0..ds {
                        let hx = ox * ds + dx;
                        let hy = oy * ds + dy;
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

        let pad_frac = (pad / ds) as f32 / out_h as f32;
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
            num_lines,
        }
    }
}

pub struct TextBox {
    pub width: Option<f32>,
    pub size: SdfSize,
    pub align: TextAlign,
    pub valign: VAlign,
    pub hyphenate: bool,
    pub spread: f32,
    pub downsample: u32,
}

impl TextBox {
    pub fn new(size: SdfSize) -> Self {
        TextBox {
            width: None,
            size,
            align: TextAlign::Left,
            valign: VAlign::Top,
            hyphenate: true,
            spread: 32.0,
            downsample: 2,
        }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn align(mut self, a: TextAlign) -> Self {
        self.align = a;
        self
    }

    pub fn valign(mut self, v: VAlign) -> Self {
        self.valign = v;
        self
    }

    pub fn hyphenate(mut self, h: bool) -> Self {
        self.hyphenate = h;
        self
    }

    pub fn spread(mut self, s: f32) -> Self {
        self.spread = s;
        self
    }

    pub fn downsample(mut self, d: u32) -> Self {
        self.downsample = d;
        self
    }
}

// --- SDF internals ---

struct SdfImage {
    image: Image,
    pad_frac: f32,
    num_lines: u32,
}

fn chamfer_distance(inside: &[bool], w: u32, h: u32) -> Vec<f32> {
    const BIG: f32 = 1e10;
    const D1: f32 = 1.0;
    const D2: f32 = 1.414;
    let n = (w * h) as usize;
    let mut dist = vec![BIG; n];
    let idx = |x: i32, y: i32| -> usize { (y as u32 * w + x as u32) as usize };

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

// --- Public entry point ---

pub fn spawn_text(
    font: &Font,
    text: &str,
    text_box: &TextBox,
    transform: Transform,
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    sdf_mats: &mut Assets<SdfTextMaterial>,
) {
    let wrapped = font.wrap(text, text_box);
    let sdf = font.generate_sdf(&wrapped, text_box);

    let w = sdf.image.width() as f32;
    let h = sdf.image.height() as f32;
    let (world_w, world_h) = match text_box.size {
        SdfSize::TotalWidth(tw) => (tw, tw * (h / w)),
        SdfSize::HeightPerLine(hpl) => {
            let content_frac = 1.0 - 2.0 * sdf.pad_frac;
            let wh = hpl * sdf.num_lines as f32 / content_frac;
            (wh * (w / h), wh)
        }
    };

    let mut transform = transform;

    if matches!(text_box.valign, VAlign::Top) {
        transform.translation.y -= world_h * (0.5 - sdf.pad_frac);
    }

    match text_box.align {
        TextAlign::Left => {
            let pad_frac_w = sdf.pad_frac * (h / w);
            let right = transform.right().as_vec3();
            transform.translation += right * world_w * (0.5 - pad_frac_w);
        }
        TextAlign::Center => {}
        TextAlign::Right => {
            let pad_frac_w = sdf.pad_frac * (h / w);
            let right = transform.right().as_vec3();
            transform.translation -= right * world_w * (0.5 - pad_frac_w);
        }
    }

    let handle = images.add(sdf.image);
    let mat = sdf_mats.add(SdfTextMaterial {
        sdf_texture: handle,
        text_color: font.color,
    });
    let mesh = meshes.add(Rectangle::new(world_w, world_h));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        transform,
        Visibility::default(),
    ));
}
