use kurbo::{Affine, Vec2};

/// Transforms a point using an affine transformation matrix.
///
/// # Parameters
/// - `affine`: The affine transformation matrix.
/// - `point`: The point to be transformed.
///
/// # Returns
/// A new point that is the result of applying the affine transformation to the original point.
///
/// # Explanation
///
/// An affine transformation is a combination of linear transformations (like scaling, rotating, or skewing)
/// and translations (moving the point around). It can be represented by a 3x3 matrix:
///
/// ```text
/// | a  b  e |
/// | c  d  f |
/// | 0  0  1 |
/// ```
///
/// The last row is always `0 0 1` and can be ignored for 2D transformations.
///
/// When you multiply this matrix with a point `(x, y)`, you get a new point `(x', y')`:
///
/// ```text
/// x' = a * x + c * y + e
/// y' = b * x + d * y + f
/// ```
///
/// # Visual Representation
///
/// Consider a point `(x, y)` and an affine transformation matrix:
///
/// ```text
/// Original Point: (x, y)
/// Affine Matrix:
/// | a  b  e |
/// | c  d  f |
/// | 0  0  1 |
///
/// Transformed Point: (x', y')
/// ```
///
/// The transformed point `(x', y')` is calculated as:
///
/// ```text
/// x' = a * x + c * y + e
/// y' = b * x + d * y + f
/// ```
///
/// # Example Usage
///
/// ```rust
/// let affine = Affine::new([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
/// let point = Vec2::new(1.0, 1.0);
/// let transformed_point = transform_point(affine, point);
/// assert_eq!(transformed_point, point);
/// ```
///
/// This example uses an identity matrix (which doesn't change the point), so the transformed point is the same as the original.
pub fn transform_point(affine: Affine, point: Vec2) -> Vec2 {
    let cooefs = affine.as_coeffs();
    let a = cooefs[0];
    let b = cooefs[1];
    let c = cooefs[2];
    let d = cooefs[3];
    let e = cooefs[4];
    let f = cooefs[5];

    let x_new = Vec2::new(a, c).dot(point) + e;
    let y_new = Vec2::new(b, d).dot(point) + f;

    Vec2::new(x_new, y_new)
}

// write doc comments for this function. make sure the comments are usefult to someone who doesn't really understand what affine transforms are. make sure the user knows what the function does and how to use it. also make sure the user knows when to use the function and why to use it and whne not to use it and why

/// Creates an affine transformation matrix that scales content to match a new aspect ratio.
///
/// # Parameters
/// - `original_aspect_ratio`: The original aspect ratio (width / height) of the content.
/// - `new_aspect_ratio`: The desired new aspect ratio (width / height) to transform to.
///
/// # Returns
/// An `Affine` transformation matrix that scales the content to match the new aspect ratio.
///
/// # Explanation
///
/// This function calculates the necessary scaling factors to adjust the aspect ratio of a 2D object.
/// The aspect ratio is defined as the ratio of width to height. For example:
///
/// ```
/// original_aspect_ratio = 4.0 / 3.0  // (width / height)
/// new_aspect_ratio = 16.0 / 9.0      // (width / height)
/// ```
///
/// To maintain the visual integrity of the object while changing its aspect ratio, we need to scale
/// it differently along the x and y axes. This is achieved by calculating the `x_scale` and `y_scale`
/// factors:
///
/// ```
/// x_scale = (new_aspect_ratio / original_aspect_ratio).min(1.0)
/// y_scale = (original_aspect_ratio / new_aspect_ratio).min(1.0)
/// ```
///
/// These scaling factors ensure that the object fits within the new aspect ratio without distortion.
///
/// # Visual Representation
///
/// Consider an object with an original aspect ratio of 4:3 that needs to be transformed to an aspect ratio of 16:9.
///
/// Original Aspect Ratio (4:3):
/// ```
/// +---------+
/// |         |
/// |         |
/// |         |
/// +---------+
/// ```
///
/// New Aspect Ratio (16:9):
/// ```
/// +-----------------+
/// |                 |
/// |                 |
/// +-----------------+
/// ```
///
/// If the content appears stretched, it means the scaling factors are not correctly applied. The function will calculate
/// the appropriate scaling factors to ensure the object fits within the new aspect ratio without stretching.
///
/// # Usage
///
/// Use this function when you need to adjust the aspect ratio of a 2D object while maintaining its visual integrity.
/// This is particularly useful in graphics applications where the display area changes and you need to adapt the content
/// to fit different screen sizes or resolutions.
///
/// # Example
///
/// ```
/// let original_aspect_ratio = 4.0 / 3.0;
/// let new_aspect_ratio = 16.0 / 9.0;
/// let affine = aspect_ratio(original_aspect_ratio, new_aspect_ratio);
/// ```
///
/// This will return an `Affine` transformation matrix that scales the object to fit the new aspect ratio.
///
/// # Alternatives
///
/// If you need to uniformly scale the content (keeping the same aspect ratio), consider using the `Affine::scale` method.
/// For more complex transformations, you may need to combine multiple affine transformations.
pub(crate) fn aspect_ratio_correction(original_aspect_ratio: f64, new_aspect_ratio: f64) -> Affine {
    let x_scale = (new_aspect_ratio / original_aspect_ratio).min(1.0);
    let y_scale = (original_aspect_ratio / new_aspect_ratio).min(1.0);
    Affine::scale_non_uniform(x_scale, y_scale)
}

pub fn aspect_ratio_correction_from_points(
    vertex_width: f64,
    vertex_height: f64,
    mandelbrot_width: f64,
    mandelbrot_height: f64,
) -> Affine {
    let scale_x = mandelbrot_width / vertex_width;
    let scale_y = mandelbrot_height / vertex_height;
    Affine::scale_non_uniform(scale_x, scale_y)
}

/// Create an affine transform to transform from one viewport to another.
///
/// # Arguments
///
/// * `original_min` - The minimum point (Vec2) of the original viewport.
/// * `original_max` - The maximum point (Vec2) of the original viewport.
/// * `new_min` - The minimum point (Vec2) of the new viewport.
/// * `new_max` - The maximum point (Vec2) of the new viewport.
///
/// # Returns
///
/// * `Affine` - The adjusted affine transform.
pub fn general_transform(
    original_min: Vec2,
    original_max: Vec2,
    new_min: Vec2,
    new_max: Vec2,
) -> Affine {
    // Calculate scale factors for the original viewport
    let scale_x = (new_max.x - new_min.x) / (original_max.x - original_min.x);
    let scale_y = (new_max.y - new_min.y) / (original_max.y - original_min.y);
    let scale_transform = Affine::scale_non_uniform(scale_x, scale_y);

    // Calculate translation factors for the new viewport
    let translate_x = new_min.x - original_min.x * scale_x;
    let translate_y = new_min.y - original_min.y * scale_y;
    let translate_transform = Affine::translate(Vec2::new(translate_x, translate_y));

    translate_transform * scale_transform
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, Comparison};
    fn _assert_near(p0: Vec2, p1: Vec2) {
        let comp = Comparison::new(&p0, &p1);
        assert!(
            (p1 - p0).hypot() < 1e-9,
            "Left expected, right found: {comp}"
        );
    }

    #[test]
    fn test_transform_point() {
        let affine = Affine::new([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        let point = Vec2::new(1.0, 1.0);
        let transformed_point = transform_point(affine, point);
        assert_eq!(transformed_point, point);
    }

    #[test]
    fn test_no_aspect_ratio_change() {
        let original_aspect_ratio = 1.0;
        let new_aspect_ratio = 1.0;
        let affine = aspect_ratio_correction(original_aspect_ratio, new_aspect_ratio);
        let original_point = Vec2::new(1.0, 1.0);
        let transformed_point = transform_point(affine, original_point);
        assert_eq!(transformed_point, original_point);
    }

    #[test]
    fn test_aspect_ratio() {
        let original_aspect_ratio = 1.0;
        let new_aspect_ratio = 2.0;
        let affine = aspect_ratio_correction(original_aspect_ratio, new_aspect_ratio);
        let original_point = Vec2::new(1.0, 1.0);
        let transformed_point = transform_point(affine, original_point);
        let expected_point = Vec2::new(1.0, 0.5);
        assert_eq!(transformed_point, expected_point);
    }
}
