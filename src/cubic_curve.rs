/// Cubic curve implementation for animation interpolation
pub struct Cubic {
    curves: Vec<f64>,
}

impl Cubic {
    /// Create a new cubic curve with the given control points
    pub fn new(curves: Vec<f64>) -> Self {
        Self { curves }
    }

    /// Calculate a value on the cubic curve for the given time parameter
    pub fn get_value(&self, time: f64) -> f64 {
        let mut start_gradient = 0.0;
        let mut end_gradient = 0.0;
        let start = 0.0;
        let mut mid = 0.0;
        let end = 1.0;

        if time <= 0.0 {
            if self.curves[0] > 0.0 {
                start_gradient = self.curves[1] / self.curves[0];
            } else if self.curves[1] == 0.0 && self.curves[2] > 0.0 {
                start_gradient = self.curves[3] / self.curves[2];
            }
            return start_gradient * time;
        }

        if time >= 1.0 {
            if self.curves[2] < 1.0 {
                end_gradient = (self.curves[3] - 1.0) / (self.curves[2] - 1.0);
            } else if self.curves[2] == 1.0 && self.curves[0] < 1.0 {
                end_gradient = (self.curves[1] - 1.0) / (self.curves[0] - 1.0);
            }
            return 1.0 + end_gradient * (time - 1.0);
        }

        let mut start_value = start;
        let mut end_value = end;

        while start_value < end_value {
            mid = (start_value + end_value) / 2.0;
            let x_est = Self::calculate(self.curves[0], self.curves[2], mid);
            if (time - x_est).abs() < 0.00001 {
                return Self::calculate(self.curves[1], self.curves[3], mid);
            }
            if x_est < time {
                start_value = mid;
            } else {
                end_value = mid;
            }
        }
        Self::calculate(self.curves[1], self.curves[3], mid)
    }

    /// Helper function to calculate points on the curve
    fn calculate(a: f64, b: f64, m: f64) -> f64 {
        3.0 * a * (1.0 - m) * (1.0 - m) * m + 3.0 * b * (1.0 - m) * m * m + m * m * m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_curve() {
        let cubic = Cubic::new(vec![0.1, 0.2, 0.3, 0.4]);
        let value = cubic.get_value(0.5);
        assert!(value > 0.0);
    }
}
