
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
    let x_new = a * point.x + c * point.y + e;
    let y_new = b * point.x + d * point.y + f;

    return vec2<f32>(x_new, y_new);
}

// Define the uniform buffer structure with aligned Affine struct
struct Globals {
    transform: Affine,
    viewport: vec2<f32>,
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
    @location(1) zoom_factor: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(in.pos, 0.0, 1.0);
    let vx = globals.viewport.x;
    let vy = globals.viewport.y;

    // it is important that this coordinate space (where the origin is and which way is growing) matches winit
    switch (in.index) {
        case 0u,3u: {
            out.uv = vec2<f32>(0.0, vy); // Bottom-left
            break;
        }
        case 1u: {
            out.uv = vec2<f32>(vx, vy); // Bottom-right
            break;
        }
        case 2u,4u: {
            out.uv = vec2<f32>(vx, 0.0); // Top-right
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

    out.uv = transform_point(globals.transform, out.uv);

    let a = globals.transform.elements[0].x;
    let b = globals.transform.elements[0].y;
    let d = globals.transform.elements[0].w;
    let zoom_factor = 1.0 / sqrt(a * a + b * b + d * d);
    out.zoom_factor = zoom_factor;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = in.uv;

    var z = vec2<f32>(0.0, 0.0);
    var i = 0u;
    let max_i = u32(100.0 * log2(in.zoom_factor));
    let epsilon = 1e-3 ; // Threshold for change in z

    loop {
        if (i >= max_i) { break; }
        if (dot(z, z) > 4.0) { break; }

        let z_new = vec2<f32>(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y + c.y
        );

        // Check if the change in z is significant
        if (length(z_new - z) < epsilon) {
            i = max_i;
            break;
        }

        z = z_new;
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
