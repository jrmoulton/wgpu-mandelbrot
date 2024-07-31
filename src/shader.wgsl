
// Define the Affine struct with proper alignment
struct Affine {
    elements: array<vec4<f32>, 2>,
};

// Function to transform a point using the Affine struct
fn transform_point(affine: Affine, point: vec2<f32>) -> vec2<f32> {
    let a = affine.elements[0].x;
    let b = affine.elements[0].y;
    let c = affine.elements[0].z;
    let d = affine.elements[0].w;
    let e = affine.elements[1].x;
    let f = affine.elements[1].y;

    // Apply the affine transformation
    let x_new = a * point.x + b * point.y + e;
    let y_new = c * point.x + d * point.y + f;

    return vec2<f32>(x_new, y_new);
}

fn normalize_translation(affine: Affine, viewport_size: vec2<f32>) -> Affine {
    let a = affine.elements[0].x;
    let b = affine.elements[0].y;
    let c = affine.elements[0].z;
    let d = affine.elements[0].w;
    let e = affine.elements[1].x;
    let f = affine.elements[1].y;

    let normalized_offset = vec2<f32>(e, f) / viewport_size;

    let a_new = a;
    let b_new = b;
    let c_new = c;
    let d_new = d;
    let e_new = normalized_offset.x;
    let f_new = normalized_offset.y;

    return Affine(
        array<vec4<f32>, 2>(
            vec4<f32>(a_new, b_new, c_new, d_new),
            vec4<f32>(e_new, f_new, 0.0, 0.0)
        )
    );
}

// Define the uniform buffer structure with aligned Affine struct
struct Globals {
    transform: Affine,
    viewport_size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

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

    let aspect_ratio = globals.viewport_size.x / globals.viewport_size.y;
    // let normalized_offset = globals.offset / globals.viewport_size;

    let transform = normalize_translation(globals.transform, globals.viewport_size);

    out.uv = transform_point(transform, out.uv);

    // adjust x coordinate for aspect ratio
    out.uv.x = out.uv.x * aspect_ratio / 2.0;
    // transform to mandelbrot coordinates
    out.uv = vec2<f32>(out.uv.x * 3.5 - 2.5 * aspect_ratio / 2.0, out.uv.y * 2.0 - 1.0);
    // out.uv = vec2<f32>(out.uv.x * 3.5 - 2.5 , out.uv.y * 2.0 - 1.0);


    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = in.uv;

    var z = vec2<f32>(0.0, 0.0);
    var i = 0u;
    let zoom_factor = 1.0 / globals.transform.elements[0].x;
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
