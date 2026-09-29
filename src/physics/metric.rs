//! Boyer–Lindquist index order: (t,r,theta,phi). Momenta are covectors.
pub type Matrix4 = [[f64; 4]; 4];
pub type Vector4 = [f64; 4];

pub fn multiply_vector(g: &Matrix4, p: &Vector4) -> Vector4 {
    std::array::from_fn(|i| g[i].iter().zip(p).map(|(a, b)| a * b).sum())
}

pub fn contract(g: &Matrix4, p: &Vector4) -> f64 {
    p.iter()
        .zip(multiply_vector(g, p))
        .map(|(a, b)| a * b)
        .sum()
}

/// Coordinate-specific metric behind a small interface; no renderer dependency.
/// A future Kerr–Schild implementation must supply its own state conversion.
pub trait Metric {
    fn covariant(&self, r: f64, theta: f64) -> Result<Matrix4, String>;
    fn inverse(&self, r: f64, theta: f64) -> Result<Matrix4, String>;
}
