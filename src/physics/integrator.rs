//! Dormand–Prince explicit embedded RK5(4). Generic ODE step, no Kerr dependence.
//! Coefficients: Dormand & Prince (1980), also SciPy RK45 / Hairer DOPRI5.
#[derive(Debug)]
pub struct Trial<const N: usize> {
    pub value: [f64; N],
    pub error: [f64; N],
}

pub fn dopri54<const N: usize>(
    y: &[f64; N],
    h: f64,
    f: impl Fn(&[f64; N]) -> Result<[f64; N], String>,
) -> Result<Trial<N>, String> {
    let mut k = [[0.0; N]; 7];
    k[0] = f(y)?;
    if !h.is_finite() || k[0].iter().any(|v| !v.is_finite()) {
        return Err("non-finite RK derivative or step".into());
    }
    let rows: [&[f64]; 6] = [
        &[1.0 / 5.0],
        &[3.0 / 40.0, 9.0 / 40.0],
        &[44.0 / 45.0, -56.0 / 15.0, 32.0 / 9.0],
        &[
            19372.0 / 6561.0,
            -25360.0 / 2187.0,
            64448.0 / 6561.0,
            -212.0 / 729.0,
        ],
        &[
            9017.0 / 3168.0,
            -355.0 / 33.0,
            46732.0 / 5247.0,
            49.0 / 176.0,
            -5103.0 / 18656.0,
        ],
        &[
            35.0 / 384.0,
            0.0,
            500.0 / 1113.0,
            125.0 / 192.0,
            -2187.0 / 6784.0,
            11.0 / 84.0,
        ],
    ];
    let mut value = *y;
    for (stage, weights) in rows.iter().enumerate() {
        value = std::array::from_fn(|i| {
            y[i] + h * weights
                .iter()
                .enumerate()
                .map(|(j, w)| w * k[j][i])
                .sum::<f64>()
        });
        if value.iter().any(|v| !v.is_finite()) {
            return Err("non-finite RK stage".into());
        }
        k[stage + 1] = f(&value)?;
        if k[stage + 1].iter().any(|v| !v.is_finite()) {
            return Err("non-finite RK derivative".into());
        }
    }
    let e = [
        35.0 / 384.0 - 5179.0 / 57600.0,
        0.0,
        500.0 / 1113.0 - 7571.0 / 16695.0,
        125.0 / 192.0 - 393.0 / 640.0,
        -2187.0 / 6784.0 + 92097.0 / 339200.0,
        11.0 / 84.0 - 187.0 / 2100.0,
        -1.0 / 40.0,
    ];
    let error =
        std::array::from_fn(|i| h * e.iter().enumerate().map(|(j, w)| w * k[j][i]).sum::<f64>());
    Ok(Trial { value, error })
}

pub fn error_norm<const N: usize>(old: &[f64; N], trial: &Trial<N>, rtol: f64, atol: f64) -> f64 {
    if trial
        .error
        .iter()
        .chain(&trial.value)
        .chain(old)
        .any(|v| !v.is_finite())
    {
        return f64::INFINITY;
    }
    (0..N)
        .map(|i| trial.error[i].abs() / (atol + rtol * old[i].abs().max(trial.value[i].abs())))
        .fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn non_finite_derivatives_and_errors_are_rejected() {
        assert!(dopri54(&[1.0], 0.1, |_| Ok([f64::NAN])).is_err());
        let bad = Trial {
            value: [1.0],
            error: [f64::NAN],
        };
        assert_eq!(error_norm(&[1.0], &bad, 1e-10, 1e-12), f64::INFINITY);
    }
    #[test]
    fn exponential_order_and_estimator() {
        let integrate = |h: f64, n: usize| {
            let mut y = [1.0];
            for _ in 0..n {
                y = dopri54(&y, h, |y| Ok(*y)).unwrap().value;
            }
            (y[0] - 1.0_f64.exp()).abs()
        };
        let coarse = integrate(0.2, 5);
        let fine = integrate(0.1, 10);
        assert!(fine < 1e-8 && coarse / fine > 20.0);
        let t = dopri54(&[1.0], 0.1, |y| Ok(*y)).unwrap();
        assert!(t.error[0].abs() > (t.value[0] - 0.1_f64.exp()).abs());
    }
}
