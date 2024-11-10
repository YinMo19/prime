use ::pyo3::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// Calculate is_prime, and return a bool.
/// use rayon to accelerate
/// use better algorithm
/// use filter
#[pyfunction]
fn is_prime(n: u64) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    let sqrt_n = (n as f64).sqrt() as u64 + 1;

    (5..sqrt_n)
        .into_par_iter()
        .map(|i| 5 + 6 * i)
        .filter(|&i| i < sqrt_n)
        .all(|i| n % i != 0 && n % (i + 2) != 0)
}

/// Calculate prime count from 2 to n, and return count.
/// use rayon to accelerate
#[pyfunction]
fn prime_count(n: usize) -> usize {
    if n < 2 {
        return 0;
    }

    let is_prime: Vec<AtomicBool> = (0..=n).map(|_| AtomicBool::new(true)).collect();
    is_prime[0].store(false, Ordering::Relaxed);
    is_prime[1].store(false, Ordering::Relaxed);

    let sqrt_n = (n as f64).sqrt() as usize;

    (2..=sqrt_n).into_par_iter().for_each(|i| {
        if is_prime[i].load(Ordering::Relaxed) {
            for j in (i * i..=n).step_by(i) {
                is_prime[j].store(false, Ordering::Relaxed);
            }
        }
    });

    is_prime
        .into_par_iter()
        .filter(|p| p.load(Ordering::Relaxed))
        .count()
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn pyo3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(is_prime, m)?)?;
    m.add_function(wrap_pyfunction!(prime_count, m)?)?;
    Ok(())
}
