use ::pyo3::prelude::*;
use rand::Rng;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

mod config; // 引入 config 模块
use config::SMALL_PRIMES;

/// Calculate is_prime, and return a bool.
/// use rayon to accelerate
/// use better algorithm
#[pyfunction]
fn is_prime(n: u128) -> bool {
    if n <= 100000 {
        return SMALL_PRIMES.contains(&n);
    }
    for &i in SMALL_PRIMES.iter() {
        if n % i == 0 {
            return false;
        }
    }

    let sqrt_n = (n as f64).sqrt() as u128 + 1;

    (0..=((sqrt_n - 5) / 6 - 100000))
        .into_par_iter()
        .flat_map(|i| vec![100005 + 6 * i, 100007 + 6 * i])
        .all(|i| n % i != 0)
}

fn miller_rabin(n: u128, k: usize) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    // 写成 n-1 = 2^s * d
    let mut s = 0;
    let mut d = n - 1;
    while d % 2 == 0 {
        d /= 2;
        s += 1;
    }

    fn trial_composite(n: u128, d: u128, s: u32, a: u128) -> bool {
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 {
            return false;
        }
        for _ in 0..(s - 1) {
            x = mod_pow(x, 2, n);
            if x == n - 1 {
                return false;
            }
        }
        true
    }

    let mut rng = rand::thread_rng();
    for _ in 0..k {
        let a = rng.gen_range(2..n - 1);
        if trial_composite(n, d, s, a) {
            return false;
        }
    }
    true
}

fn mod_pow(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % modulus;
        }
        exp >>= 1;
        base = base * base % modulus;
    }
    result
}

fn lucas_test(n: u128) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    fn find_d(n: u128) -> i128 {
        let mut d = 5;
        while (d * d - 4) % n == 0 {
            d += 2;
        }
        d as i128
    }

    let d = find_d(n);
    let p = 1;
    let q = (1 - d) / 4;

    fn lucas_sequence(p: i128, q: i128, n: u128, k: u128) -> (u128, u128, u128) {
        let d = p * p - 4 * q;
        let mut u = 0;
        let mut v = 2;
        let mut qk = q as u128;

        for bit in format!("{:b}", k).chars() {
            if bit == '1' {
                let new_u = (u * v * p as u128 + (v * v - u * u) * qk) % n;
                let new_v = (v * v + u * u * d as u128) % n;
                u = new_u;
                v = new_v;
                qk = (qk * qk) % n;
            } else {
                let new_u = (u * v * p as u128 - (v * v - u * u) * qk) % n;
                let new_v = (v * v - u * u * d as u128) % n;
                u = new_u;
                v = new_v;
                qk = (qk * qk) % n;
            }
        }
        (u, v, qk)
    }

    let (u, _, _) = lucas_sequence(p, q, n, n + 1);
    u == 0
}

fn bpsw_test(n: u128) -> bool {
    if !miller_rabin(n, 1) {
        return false;
    }
    lucas_test(n)
}

#[pyfunction]
fn is_prime_unsafe(n: u128) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    // 对于小数，使用确定性的 Miller-Rabin 测试
    if n < 2u128.pow(64) {
        return miller_rabin(n, 5);
    }

    // 对于大数，使用 BPSW 测试
    bpsw_test(n)
}

fn gen_prime_list(n: usize) -> Result<Vec<AtomicBool>, Box<dyn std::error::Error>> {
    if n < 2 {
        return Err("n must be greater than or equal to 2.".into());
    }

    let is_prime_list: Vec<AtomicBool> = (0..=n).map(|_| AtomicBool::new(true)).collect();
    is_prime_list[0].store(false, Ordering::Relaxed);
    is_prime_list[1].store(false, Ordering::Relaxed);

    let sqrt_n = (n as f64).sqrt() as usize;

    (2..=sqrt_n).into_par_iter().for_each(|i| {
        if is_prime_list[i].load(Ordering::Relaxed) {
            for j in (i * i..=n).step_by(i) {
                is_prime_list[j].store(false, Ordering::Relaxed);
            }
        }
    });

    Ok(is_prime_list)
}

/// Calculate prime count from 2 to n, and return count.
/// use rayon to accelerate
#[pyfunction]
fn prime_count(n: usize) -> usize {
    let is_prime_list = gen_prime_list(n).expect("Failed to generate prime list.");

    is_prime_list
        .into_par_iter()
        .filter(|p| p.load(Ordering::Relaxed))
        .count()
}

/// Calculate prime count from 2 to n, and return count.
/// use rayon to accelerate
#[pyfunction]
fn prime_count_range(a: usize, n: usize) -> usize {
    // if n - a < 10000, use simple algorithm
    if n - a < 10000 {
        return (a..=n)
            .into_par_iter()
            .filter(|&i| is_prime(i as u128))
            .count();
    }

    // else use filter algorithm
    let is_prime_list = gen_prime_list(n).expect("Failed to generate prime list.");

    is_prime_list
        .into_par_iter()
        .skip(a - 1)
        .filter(|p| p.load(Ordering::Relaxed))
        .count()
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn prime(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(is_prime, m)?)?;
    m.add_function(wrap_pyfunction!(is_prime_unsafe, m)?)?;
    m.add_function(wrap_pyfunction!(prime_count, m)?)?;
    m.add_function(wrap_pyfunction!(prime_count_range, m)?)?;
    Ok(())
}
