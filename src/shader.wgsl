
// Define uniform buffer structure
struct Uniforms {
    scale_low: f32,
    scale_high: f32,
    offset_low: vec2<f32>,
    offset_high: vec2<f32>,
    viewport_size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(in.pos, 0.0, 1.0);

    switch (in.index) {
        case 0u,3u: {
            out.uv = vec2<f32>(0.0, 1.0); // Bottom-left
            break;
        }
        case 1u: {
            out.uv = vec2<f32>(1.0, 1.0); // Bottom-right
            break;
        }
        case 2u,4u: {
            out.uv = vec2<f32>(1.0, 0.0); // Top-right
            break;
        }
        case 5u: {
            out.uv = vec2<f32>(0.0, 0.0); // Top-left
            break;
        }
        default: {
            out.uv = vec2<f32>(0.0, 0.0); // Default case
        }
    }

    // Calculate aspect ratio
    let aspect_ratio = uniforms.viewport_size.x / uniforms.viewport_size.y;


    // Adjust UV coordinates based on the uniform scale and offset
    out.uv = out.uv / uniforms.scale - uniforms.offset / uniforms.scale / uniforms.viewport_size;
    out.uv = vec2<f32>(out.uv.x * 3.5 - 2.5, out.uv.y * 2.0 - 1.0);

    out.uv = vec2<f32>(out.uv.x * aspect_ratio / 2.0, out.uv.y);
    // // Adjust UV coordinates based on the aspect ratio to fill the entire viewport
    // if (aspect_ratio > 1.0) {
    //     // Width > Height, scale UVs horizontally
    // } else {
    //     // Height > Width, scale UVs vertically
    //     out.uv = vec2<f32>(out.uv.x, out.uv.y / aspect_ratio);
    // }

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = in.uv;

    var z = vec2<f32>(0.0, 0.0);
    var i = 0u;
    let zoom_factor = uniforms.scale;  // Assuming scale < 1 means zooming in
    let max_i = 100u + u32(300.0 * log2(zoom_factor));

    loop {
        if (i >= max_i) { break; }
        if (dot(z, z) > 4.0) { break; }
        z = vec2<f32>(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y + c.y
        );
        i += 1u;
    }

    let t = f32(i) / f32(max_i);
    let color = vec4<f32>(
        0.5 + 0.5 * cos(3.0 + 6.28318 * t),
        0.5 + 0.5 * cos(3.0 + 6.28318 * t + 2.0),
        0.5 + 0.5 * cos(3.0 + 6.28318 * t + 4.0),
        1.0
    );
    return color;
}
