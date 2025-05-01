use crate::error::Error;

/// Interpolate between two lists of numerical values
pub fn interpolate(from_list: &[f64], to_list: &[f64], f: f64) -> Result<Vec<f64>, Error> {
    if from_list.len() != to_list.len() {
        return Err(Error::MismatchedArguments);
    }

    let mut out = Vec::with_capacity(from_list.len());
    for i in 0..from_list.len() {
        out.push(interpolate_num(from_list[i], to_list[i], f));
    }
    Ok(out)
}

/// Interpolate between two numerical values
pub fn interpolate_num(from_val: f64, to_val: f64, f: f64) -> f64 {
    from_val * (1.0 - f) + to_val * f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate() {
        let from = vec![0.0, 10.0, 20.0];
        let to = vec![100.0, 110.0, 120.0];
        let result = interpolate(&from, &to, 0.5).unwrap();
        assert_eq!(result, vec![50.0, 60.0, 70.0]);
    }

    #[test]
    fn test_interpolate_error() {
        let from = vec![0.0, 10.0];
        let to = vec![100.0, 110.0, 120.0];
        let result = interpolate(&from, &to, 0.5);
        assert!(result.is_err());
    }
}
