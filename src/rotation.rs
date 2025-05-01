use std::f64::consts::PI;

/// Convert rotation from degrees to a 2D rotation matrix
pub fn convert_rotation_to_matrix(degrees: f64) -> Vec<f64> {
    let radians = degrees * PI / 180.0;
    let cos = radians.cos();
    let sin = radians.sin();
    vec![cos, -sin, sin, cos]
}

/// Convert rotation from degrees to a 2D transformation matrix (expanded form)
#[allow(dead_code)]
pub fn convert_rotation_to_transform_matrix(degrees: f64) -> Vec<f64> {
    let radians = degrees * PI / 180.0;
    let cos = radians.cos();
    let sin = radians.sin();
    vec![cos, sin, -sin, cos, 0.0, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_matrix() {
        let matrix = convert_rotation_to_matrix(90.0);
        assert!((matrix[0] - 0.0).abs() < 0.00001); // cos(90°) ≈ 0
        assert!((matrix[1] - (-1.0)).abs() < 0.00001); // -sin(90°) = -1
        assert!((matrix[2] - 1.0).abs() < 0.00001); // sin(90°) = 1
        assert!((matrix[3] - 0.0).abs() < 0.00001); // cos(90°) ≈ 0
    }

    #[test]
    fn test_transform_matrix() {
        let matrix = convert_rotation_to_transform_matrix(90.0);
        assert!((matrix[0] - 0.0).abs() < 0.00001); // cos(90°) ≈ 0
        assert!((matrix[1] - 1.0).abs() < 0.00001); // sin(90°) = 1
        assert!((matrix[2] - (-1.0)).abs() < 0.00001); // -sin(90°) = -1
        assert!((matrix[3] - 0.0).abs() < 0.00001); // cos(90°) ≈ 0
        assert_eq!(matrix[4], 0.0);
        assert_eq!(matrix[5], 0.0);
    }
}
