//! Natural units. Changing M alone would break the dimensionless API.
pub const MASS: f64 = 1.0;
/// Supported signed-spin magnitude. Exact extremality remains excluded.
/// This domain guard is not a guarantee of numerical accuracy for every ray.
pub const MAX_SPIN: f64 = 0.9999;
pub const EQUATOR: f64 = std::f64::consts::FRAC_PI_2;
