//! Quant-interview staples: fraction/percent fluency, fast estimation,
//! squares, powers of two and expected value.

use rand::RngExt;

use super::{Method, Technique, n_digits, pick};
use crate::problem::{Answer, Expr, Problem, fmt_num};

pub const TECHNIQUES: &[Technique] = &[
    Technique {
        name: "Fraction → percent",
        rule: &[
            "Memorise the unit fractions: 1/6 = 16.67%, 1/7 = 14.29%, 1/8 = 12.5%, 1/9 = 11.11%, 1/11 = 9.09%, 1/12 = 8.33%, 1/16 = 6.25%.",
            "Then n/d = n × (1/d).",
        ],
    },
    Technique {
        name: "Percent of a number",
        rule: &[
            "Build from 10% (move the decimal), 5% (half of 10%) and 1%.",
            "Example: 17% of 340 = 34 + 17 + 2×3.4 = 57.8.",
        ],
    },
    Technique {
        name: "Estimate a product",
        rule: &[
            "Round to 2 significant figures and multiply.",
            "Need more precision? Correct with the rounding error: a × b = a' × b + (a − a') × b.",
        ],
    },
    Technique {
        name: "Estimate a quotient",
        rule: &[
            "Round the divisor to something easy, divide, then scale by (rounded divisor ÷ true divisor).",
            "Dividing by b' instead of b ≈ multiplying by 1 + (b' − b)/b.",
        ],
    },
    Technique {
        name: "Squares to 150",
        rule: &[
            "n² = (n − k)(n + k) + k², with k chosen so one factor is a round number.",
            "Example: 47² = 44 × 50 + 3² = 2200 + 9 = 2209.",
        ],
    },
    Technique {
        name: "Powers of 2",
        rule: &[
            "Anchors: 2^5 = 32, 2^10 = 1024, 2^16 = 65536, 2^20 = 1048576, 2^24 = 16777216, 2^30 = 1073741824.",
            "Start from the nearest anchor below and double.",
        ],
    },
    Technique {
        name: "Expected value of a bet",
        rule: &["EV = p × win − (1 − p) × loss. Put both over the common denominator first."],
    },
];

fn problem(t: usize, prompt: String, answer: Answer, expr: Expr, steps: Vec<String>) -> Problem {
    Problem {
        method: Method::Quant,
        technique: t,
        prompt,
        answer,
        expr,
        steps,
        flash: None,
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

/// True when n/d has a terminating decimal expansion.
fn terminates(d: i64) -> bool {
    let mut d = d;
    for p in [2, 5] {
        while d % p == 0 {
            d /= p;
        }
    }
    d == 1
}

/// Exact if the decimal terminates, otherwise rounded to 2 dp.
fn ratio_answer(num: i64, den: i64) -> Answer {
    let v = num as f64 / den as f64;
    let g = gcd(num, den);
    if terminates(den / g) {
        Answer::exact(v)
    } else {
        Answer::rounded(v, 2)
    }
}

fn round_sig2(n: i64) -> i64 {
    let digits = n.to_string().len() as u32;
    if digits <= 2 {
        return n;
    }
    let p = 10i64.pow(digits - 2);
    (n + p / 2) / p * p
}

fn round_sig1(n: i64) -> i64 {
    let digits = n.to_string().len() as u32;
    let p = 10i64.pow(digits - 1);
    (n + p / 2) / p * p
}

fn pct_off(est: f64, exact: f64) -> f64 {
    (est - exact).abs() / exact.abs() * 100.0
}

pub fn generate(t: usize, diff: u8, rng: &mut impl RngExt) -> Problem {
    match t {
        0 => {
            let dens: &[i64] = match diff {
                1 => &[2, 4, 5, 8, 10],
                2 | 3 => &[2, 3, 4, 5, 6, 8, 9, 12, 16, 20],
                _ => &[3, 6, 7, 8, 9, 11, 12, 15, 16, 25, 40],
            };
            let d = dens[rng.random_range(0..dens.len())];
            let n = loop {
                let n = rng.random_range(1..d);
                if gcd(n, d) == 1 {
                    break n;
                }
            };
            let ans = ratio_answer(100 * n, d);
            let unit = 100.0 / d as f64;
            let approx = if terminates(d) { "=" } else { "≈" };
            let steps = vec![
                format!("Anchor: 1/{d} {approx} {}%", fmt_num(unit, 4)),
                format!("{n}/{d} = {n} × {}% {approx} {}%", fmt_num(unit, 4), ans.shown),
            ];
            problem(
                t,
                format!("{n}/{d} as a percent"),
                ans,
                Expr::Ratio(100 * n, d),
                steps,
            )
        }
        1 => {
            let (p, x) = match diff {
                1 => (5 * rng.random_range(1..=10), 10 * rng.random_range(2..=50)),
                2 => (5 * rng.random_range(1..=19), 10 * rng.random_range(2..=99)),
                _ => (rng.random_range(1..=99), rng.random_range(20..=999)),
            };
            let ans = Answer::exact((p * x) as f64 / 100.0);
            let ten = x as f64 / 10.0;
            let (tens, five, ones) = (p / 10, (p % 10) / 5, p % 5);
            let mut steps = vec![format!("10% of {x} = {}", fmt_num(ten, 4))];
            let mut parts: Vec<f64> = vec![];
            if tens > 0 {
                let v = ten * tens as f64;
                steps.push(format!(
                    "{}% = {tens} × {} = {}",
                    tens * 10,
                    fmt_num(ten, 4),
                    fmt_num(v, 4)
                ));
                parts.push(v);
            }
            if five > 0 {
                let v = ten / 2.0;
                steps.push(format!("5% = half of 10% = {}", fmt_num(v, 4)));
                parts.push(v);
            }
            if ones > 0 {
                let one = x as f64 / 100.0;
                let v = one * ones as f64;
                steps.push(format!(
                    "1% = {}, so {ones}% = {}",
                    fmt_num(one, 4),
                    fmt_num(v, 4)
                ));
                parts.push(v);
            }
            let joined = parts
                .iter()
                .map(|v| fmt_num(*v, 4))
                .collect::<Vec<_>>()
                .join(" + ");
            steps.push(format!("{p}% of {x} = {joined} = {}", ans.shown));
            problem(t, format!("{p}% of {x}"), ans, Expr::Ratio(p * x, 100), steps)
        }
        2 => {
            let (da, db, tol) = match diff {
                1 => (2, 2, 0.05),
                2 => (3, 2, 0.05),
                3 => (3, 2, 0.02),
                4 => (3, 3, 0.02),
                _ => (4, 3, 0.01),
            };
            let (a, b) = (n_digits(da, rng), n_digits(db, rng));
            let exact = (a * b) as f64;
            let ans = Answer::approx(exact, tol);
            let (a1, b1) = (round_sig2(a), round_sig2(b));
            let e1 = a1 * b1;
            let mut steps = vec![
                format!("Round to 2 significant figures: {a} ≈ {a1}, {b} ≈ {b1}"),
                format!(
                    "{a1} × {b1} = {e1}  ({}% off)",
                    fmt_num(pct_off(e1 as f64, exact), 1)
                ),
            ];
            let tol_pct = tol * 100.0;
            if pct_off(e1 as f64, exact) > tol_pct {
                let e2 = a * b1;
                steps.push(format!(
                    "Outside {}%, so correct for {a} vs {a1}: ({}) × {b1} = {} → {e2}  ({}% off)",
                    fmt_num(tol_pct, 0),
                    a - a1,
                    (a - a1) * b1,
                    fmt_num(pct_off(e2 as f64, exact), 2)
                ));
                if pct_off(e2 as f64, exact) > tol_pct {
                    steps.push(format!(
                        "Correct for {b} vs {b1}: {a} × ({}) = {} → {}",
                        b - b1,
                        a * (b - b1),
                        a * b
                    ));
                }
            }
            steps.push(format!("Exact: {}", ans.shown));
            problem(t, format!("Estimate {a} × {b}"), ans, Expr::Mul(a, b), steps)
        }
        3 => {
            let (dd, dv, tol) = match diff {
                1 => (3, 2, 0.05),
                2 => (4, 2, 0.05),
                3 => (4, 2, 0.02),
                4 => (5, 2, 0.02),
                _ => (6, 3, 0.02),
            };
            let (a, b) = (n_digits(dd, rng), n_digits(dv, rng));
            let exact = a as f64 / b as f64;
            let ans = Answer::approx(exact, tol);
            let b1 = round_sig1(b);
            let mut steps = vec![];
            if b1 == b {
                steps.push(format!("{b} is already round: {a} ÷ {b} = {}", fmt_num(exact, 2)));
            } else {
                let q1 = a as f64 / b1 as f64;
                let factor = b1 as f64 / b as f64;
                steps.push(format!("Round the divisor: {b} ≈ {b1}"));
                steps.push(format!(
                    "{a} ÷ {b1} = {}  ({}% off)",
                    fmt_num(q1, 2),
                    fmt_num(pct_off(q1, exact), 1)
                ));
                steps.push(format!(
                    "Scale by {b1}/{b} ≈ 1 {} {} → {}",
                    if factor >= 1.0 { "+" } else { "−" },
                    fmt_num((factor - 1.0).abs(), 3),
                    fmt_num(q1 * factor, 2)
                ));
            }
            steps.push(format!("Exact: {}", ans.shown));
            problem(t, format!("Estimate {a} ÷ {b}"), ans, Expr::Ratio(a, b), steps)
        }
        4 => {
            let n: i64 = loop {
                let n = match diff {
                    1 => rng.random_range(11..=25),
                    2 => rng.random_range(11..=40),
                    3 => rng.random_range(11..=60),
                    4 => rng.random_range(11..=99),
                    _ => rng.random_range(101..=150),
                };
                if n % 10 != 0 {
                    break n;
                }
            };
            let m = (n + 5) / 10 * 10;
            let k = (n - m).abs();
            let (lo, hi) = (n - k, n + k);
            let result = n * n;
            let steps = vec![
                format!("{n}² = ({n} − {k})({n} + {k}) + {k}² = {lo} × {hi} + {}", k * k),
                format!("{lo} × {hi} = {}", lo * hi),
                format!("{} + {} = {result}", lo * hi, k * k),
            ];
            problem(t, format!("{n}²"), Answer::int(result), Expr::Mul(n, n), steps)
        }
        5 => {
            let (lo, hi) = match diff {
                1 => (1, 10),
                2 => (5, 14),
                3 => (8, 18),
                4 => (10, 24),
                _ => (15, 30),
            };
            let k: u32 = rng.random_range(lo..=hi);
            let anchor = [30u32, 24, 20, 16, 10, 5, 0]
                .into_iter()
                .find(|&a| a <= k)
                .unwrap();
            let mut v = 1i64 << anchor;
            let mut chain = vec![v.to_string()];
            for _ in anchor..k {
                v *= 2;
                chain.push(v.to_string());
            }
            let steps = vec![
                format!("Anchor 2^{anchor} = {}", 1i64 << anchor),
                format!("Double {} time(s): {}", k - anchor, chain.join(" → ")),
                format!("2^{k} = {v}"),
            ];
            problem(t, format!("2^{k}"), Answer::int(v), Expr::Pow2(k), steps)
        }
        _ => {
            let dens: &[i64] = if diff == 1 {
                &[2, 4, 5, 10]
            } else {
                &[2, 3, 4, 5, 6, 8, 10]
            };
            let den = dens[rng.random_range(0..dens.len())];
            let num = rng.random_range(1..den);
            let scale = 5 * diff as i64;
            let win = scale * rng.random_range(1..=10);
            let lose = scale * rng.random_range(1..=10);
            let numer = num * win - (den - num) * lose;
            let ans = ratio_answer(numer, den);
            let prompt = if den == 6 {
                format!("Roll a die: win ${win} on {num} of 6 faces, else lose ${lose}. EV?")
            } else {
                format!("Win ${win} with prob {num}/{den}, else lose ${lose}. EV?")
            };
            let steps = vec![
                "EV = P(win) × win − P(lose) × loss".to_string(),
                format!("= ({num}/{den}) × {win} − ({}/{den}) × {lose}", den - num),
                format!(
                    "= ({} − {}) / {den} = {numer}/{den}",
                    num * win,
                    (den - num) * lose
                ),
                format!("= {}", ans.shown),
            ];
            problem(t, prompt, ans, Expr::Ev { num, den, win, lose }, steps)
        }
    }
}

pub fn generate_random(diff: u8, rng: &mut impl RngExt) -> Problem {
    generate(pick(TECHNIQUES.len(), rng), diff, rng)
}
