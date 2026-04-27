#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals

@fragment
fn fragment0(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1., 0.5, 1., 1.);
}

@fragment
fn fragment1(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.uv.x, in.uv.y, sin(globals.time), 1.);
}
