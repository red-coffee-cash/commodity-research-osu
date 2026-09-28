//! Vedic and base-number shortcuts: squares near a base, products near 100,
//! difference of squares and multiplying by 9s and 25.

use rand::RngExt;

use super::{Method, Technique, n_digits, pick};
use crate::problem::{Answer, Expr, Problem};

pub const TECHNIQUES: &[Technique] = &[
    Technique {
        name: "Squares ending in 5",
        rule: &["(10a + 5)² = a × (a + 1), then append 25.  Example: 65² → 6 × 7 = 42 → 4225."],
    },
    Technique {
        name: "Squares near 50",
        rule: &["(50 + d)² = (25 + d) hundreds + d².  Example: 47² → d = −3 → 22 hundreds + 9 = 2209."],
    },
    Technique {
        name: "Squares near 100",
        rule: &[
            "(100 + d)² = (100 + 2d) hundreds + d².  So add d to the number, ×100, then add d².",
            "Example: 96² → d = −4 → 92 hundreds + 16 = 9216.",
        ],
    },
    Technique {
        name: "Products near 100 (Nikhilam)",
        rule: &[
            "Write each number's deviation from 100: a = 100 + x, b = 100 + y.",
            "a × b = (a + y) hundreds + x × y.  Example: 97 × 94 → (97 − 6) = 91 hundreds + 18 = 9118.",
        ],
    },
    Technique {
        name: "Difference of squares",
        rule: &[
            "If two numbers sit k either side of a round m: (m − k)(m + k) = m² − k².",
            "Example: 47 × 53 = 50² − 3² = 2500 − 9 = 2491.",
        ],
    },
    Technique {
        name: "×9, ×99, ×999",
        rule: &["99…9 = 10ᵏ − 1, so n × 99 = n × 100 − n.  Example: 47 × 99 = 4700 − 47 = 4653."],
    },
    Technique {
        name: "×25",
        rule: &[
            "25 = 100 ÷ 4: divide by 4, then ×100. A remainder r becomes r × 25.",
            "Example: 38 × 25 → 38 ÷ 4 = 9 r 2 → 900 + 50 = 950.",
        ],
    },
    Technique {
        name: "Any 2-digit square",
        rule: &[
            "Split n = h + l with h round: n² = h² + 2hl + l².",
            "Example: 73² = 70² + 2·70·3 + 3² = 4900 + 420 + 9 = 5329.",
        ],
    },
];

/// "+ 3" or "− 3".
fn pm(d: i64) -> String {
    if d < 0 {
        format!("− {}", -d)
    } else {
        format!("+ {d}")
    }
}

fn nonzero(range: i64, rng: &mut impl RngExt) -> i64 {
    let d = rng.random_range(1..=range);
    if rng.random_bool(0.5) { d } else { -d }
}

fn problem(t: usize, prompt: String, result: i64, expr: Expr, steps: Vec<String>) -> Problem {
    Problem {
        method: Method::Vedic,
        technique: t,
        prompt,
        answer: Answer::int(result),
        expr,
        steps,
        flash: None,
    }
}

pub fn generate(t: usize, diff: u8, rng: &mut impl RngExt) -> Problem {
    let diff = diff as i64;
    match t {
        0 => {
            let (lo, hi) = match diff {
                1 => (1, 9),
                2 => (1, 12),
                3 => (1, 15),
                4 => (1, 19),
                _ => (10, 29),
            };
            let a = rng.random_range(lo..=hi);
            let n = 10 * a + 5;
            let p = a * (a + 1);
            let result = p * 100 + 25;
            let steps = vec![
                format!("{n} ends in 5; the front part is a = {a}"),
                format!("a × (a + 1) = {a} × {} = {p}", a + 1),
                format!("Append 25: {n}² = {result}"),
            ];
            problem(t, format!("{n}²"), result, Expr::Mul(n, n), steps)
        }
        1 => {
            let d = nonzero(diff + 2, rng);
            let n = 50 + d;
            let h = 25 + d;
            let result = h * 100 + d * d;
            let steps = vec![
                format!("{n} = 50 {}", pm(d)),
                format!("Hundreds: 25 {} = {h} → {}", pm(d), h * 100),
                format!("Add d² = {}² = {}", d.abs(), d * d),
                format!("{} + {} = {result}", h * 100, d * d),
            ];
            problem(t, format!("{n}²"), result, Expr::Mul(n, n), steps)
        }
        2 => {
            let d = nonzero(2 + 2 * diff, rng);
            let n = 100 + d;
            let h = n + d;
            let result = h * 100 + d * d;
            let steps = vec![
                format!("{n} = 100 {}", pm(d)),
                format!(
                    "Add the deviation to the number: {n} {} = {h} → {}",
                    pm(d),
                    h * 100
                ),
                format!("Add d² = {}² = {}", d.abs(), d * d),
                format!("{} + {} = {result}", h * 100, d * d),
            ];
            problem(t, format!("{n}²"), result, Expr::Mul(n, n), steps)
        }
        3 => {
            let (x, y) = (nonzero(1 + 2 * diff, rng), nonzero(1 + 2 * diff, rng));
            let (a, b) = (100 + x, 100 + y);
            let h = a + y;
            let xy = x * y;
            let result = h * 100 + xy;
            let steps = vec![
                format!("Deviations from 100: {a} → {x:+}, {b} → {y:+}"),
                format!(
                    "Cross-add: {a} {} = {h} (same as {b} {}) → {}",
                    pm(y),
                    pm(x),
                    h * 100
                ),
                format!("Multiply the deviations: ({x}) × ({y}) = {xy}"),
                format!("{} {} = {result}", h * 100, pm(xy)),
            ];
            problem(t, format!("{a} × {b}"), result, Expr::Mul(a, b), steps)
        }
        4 => {
            let m = match diff {
                1 | 2 => 10 * rng.random_range(2..=9),
                3 => 10 * rng.random_range(2..=15),
                _ => 5 * rng.random_range(4..=40),
            };
            let k = rng.random_range(1..=diff + 2);
            let (mut a, mut b) = (m - k, m + k);
            if rng.random_bool(0.5) {
                std::mem::swap(&mut a, &mut b);
            }
            let result = m * m - k * k;
            let steps = vec![
                format!("{a} and {b} are both {k} away from {m}, so the product is {m}² − {k}²"),
                format!("{m}² = {}", m * m),
                format!("{k}² = {}", k * k),
                format!("{} − {} = {result}", m * m, k * k),
            ];
            problem(t, format!("{a} × {b}"), result, Expr::Mul(a, b), steps)
        }
        5 => {
            let k = rng.random_range(1..=3u32);
            let p = 10i64.pow(k);
            let nines = p - 1;
            let n = n_digits((diff as u32 + 1).min(5), rng);
            let result = n * p - n;
            let steps = vec![
                format!("{nines} = {p} − 1"),
                format!("{n} × {p} = {}", n * p),
                format!("{} − {n} = {result}", n * p),
            ];
            problem(t, format!("{n} × {nines}"), result, Expr::Mul(n, nines), steps)
        }
        6 => {
            let n = if diff == 1 {
                4 * rng.random_range(3..=24)
            } else {
                n_digits((diff as u32 + 1).min(5), rng)
            };
            let (q, r) = (n / 4, n % 4);
            let result = q * 100 + r * 25;
            let steps = vec![
                "×25 is ×100 then ÷4, so divide by 4 first".to_string(),
                format!("{n} ÷ 4 = {q} remainder {r}"),
                format!("{q} × 100 = {}; remainder {r} × 25 = {}", q * 100, r * 25),
                format!("{} + {} = {result}", q * 100, r * 25),
            ];
            problem(t, format!("{n} × 25"), result, Expr::Mul(n, 25), steps)
        }
        _ => {
            let (n, h) = loop {
                let n = match diff {
                    1 => rng.random_range(11..=39),
                    2 => rng.random_range(11..=59),
                    3 | 4 => rng.random_range(11..=99),
                    _ => rng.random_range(101..=199),
                };
                let h = if n > 100 { 100 } else { n / 10 * 10 };
                if n != h {
                    break (n, h);
                }
            };
            let l = n - h;
            let result = n * n;
            let steps = vec![
                format!("{n} = {h} + {l}"),
                format!("{h}² = {}", h * h),
                format!("2 × {h} × {l} = {}", 2 * h * l),
                format!("{l}² = {}", l * l),
                format!("{} + {} + {} = {result}", h * h, 2 * h * l, l * l),
            ];
            problem(t, format!("{n}²"), result, Expr::Mul(n, n), steps)
        }
    }
}

pub fn generate_random(diff: u8, rng: &mut impl RngExt) -> Problem {
    generate(pick(TECHNIQUES.len(), rng), diff, rng)
}
