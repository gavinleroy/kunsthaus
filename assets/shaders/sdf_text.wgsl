#import bevy_pbr::forward_io::VertexOutput

@group(3) @binding(0) var sdf_texture: texture_2d<f32>;
@group(3) @binding(1) var sdf_sampler: sampler;

struct TextColor {
    color: vec4<f32>,
};
@group(3) @binding(2) var<uniform> text_color: TextColor;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let sdf_value = textureSample(sdf_texture, sdf_sampler, in.uv).a;
    let fw = fwidth(sdf_value) * 0.5;
    let alpha = smoothstep(0.5 - fw, 0.5 + fw, sdf_value);
    let color = text_color.color;
    return vec4<f32>(color.rgb, color.a * alpha);
}
