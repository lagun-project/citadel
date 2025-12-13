// Point shader for instanced node rendering.
// Each instance represents a single mesh node.

struct CameraUniform {
    view_proj: mat4x4<f32>,
    point_size: f32,
    mesh_alpha: f32,  // 1.0 = opaque, 0.0 = invisible
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) mesh_alpha: f32,
}

// Unpack RGBA from u32 (ABGR format)
fn unpack_color(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = f32((packed >> 24u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

@vertex
fn vs_main(instance: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform position to clip space
    out.clip_position = camera.view_proj * vec4<f32>(instance.position, 1.0);

    // Unpack color
    out.color = unpack_color(instance.color);

    // Pass through mesh alpha
    out.mesh_alpha = camera.mesh_alpha;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Apply mesh alpha to make mesh transparent when path visualization is active
    return vec4<f32>(in.color.rgb, in.color.a * in.mesh_alpha);
}
