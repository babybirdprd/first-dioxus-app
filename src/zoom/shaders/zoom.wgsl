struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    zoom: f32,
    center_x: f32,
    center_y: f32,
    aspect: f32,
    blur_samples: f32,
    prev_center_x: f32,
    prev_center_y: f32,
    prev_zoom: f32,
    width: f32,
    height: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Using a switch to avoid "may only be indexed by a constant" error on some backends
    var p: vec2<f32>;
    switch (vertex_index) {
        case 0u: { p = vec2<f32>(-1.0, -1.0); } // Bottom Left
        case 1u: { p = vec2<f32>( 1.0, -1.0); } // Bottom Right
        case 2u: { p = vec2<f32>(-1.0,  1.0); } // Top Left
        case 3u: { p = vec2<f32>(-1.0,  1.0); } // Top Left
        case 4u: { p = vec2<f32>( 1.0, -1.0); } // Bottom Right
        case 5u: { p = vec2<f32>( 1.0,  1.0); } // Top Right
        default: { p = vec2<f32>(0.0, 0.0); }
    }
    
    out.position = vec4<f32>(p, 0.0, 1.0);
    
    // Texture coordinates: p.x (-1 to 1) -> (0 to 1), p.y (-1 to 1) -> (1 to 0)
    // p.y = 1 (top) -> tex_y = 0
    // p.y = -1 (bottom) -> tex_y = 1
    out.tex_coords = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

fn get_sampled_coords(tex_coords: vec2<f32>, zoom: f32, cx: f32, cy: f32) -> vec2<f32> {
    let inv_zoom = 1.0 / zoom;
    let half_size = inv_zoom * 0.5;
    
    // Clamp center so we don't sample outside
    let clamp_cx = clamp(cx, half_size, 1.0 - half_size);
    let clamp_cy = clamp(cy, half_size, 1.0 - half_size);
    
    let start_x = clamp_cx - half_size;
    let start_y = clamp_cy - half_size;
    
    return vec2<f32>(
        start_x + tex_coords.x * inv_zoom,
        start_y + tex_coords.y * inv_zoom
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (u.blur_samples <= 1.0) {
        let coords = get_sampled_coords(in.tex_coords, u.zoom, u.center_x, u.center_y);
        return textureSample(t_diffuse, s_diffuse, coords);
    }

    var total_color = vec4<f32>(0.0);
    let samples = i32(u.blur_samples);
    
    for (var i = 0; i < samples; i = i + 1) {
        let t = f32(i) / f32(samples - 1);
        
        let cur_zoom = mix(u.prev_zoom, u.zoom, t);
        let cur_cx = mix(u.prev_center_x, u.center_x, t);
        let cur_cy = mix(u.prev_center_y, u.center_y, t);
        
        let coords = get_sampled_coords(in.tex_coords, cur_zoom, cur_cx, cur_cy);
        total_color = total_color + textureSample(t_diffuse, s_diffuse, coords);
    }
    
    return total_color / f32(samples);
}
