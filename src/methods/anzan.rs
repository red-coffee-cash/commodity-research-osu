//! Flash anzan: numbers are flashed one at a time and you keep a running total
//! on an imagined soroban. Worked steps for single digits spell out the bead
//! moves, including the 5- and 10-complements.

use rand::RngExt;

use super::{Method, Technique, n_digits};
use crate::problem::{Answer, Expr, Problem};

pub const TECHNIQUES: &[Technique] = &[
    Technique {
        name: "1-digit, soroban complements",
        rule: &[
            "Keep the total on an imagined soroban. Units rod: heaven bead = 5, earth beads = 1 each.",
            "Direct: if the beads are free, just move them.",
            "5-complement (+a when earth beads are short): lower the heaven bead (+5), drop 5−a earth.",
            "10-complement (+a overflows the rod): remove 10−a on this rod, add 1 on the tens rod.",
        ],
    },
    Technique {
        name: "Multi-digit additions",
        rule: &[
            "Add from the left, rod by rod (tens first, then units), updating the picture.",
            "Say only the running total to yourself, never the sum being added.",
        ],
    },
    Technique {
        name: "Additions and subtractions",
        rule: &[
            "A minus sign means take away on the soroban; use complements in reverse:",
            "−a with too few beads: remove 10 (1 from the next rod), add back 10−a.",
        ],
    },
];

/// Default flash interval per difficulty, in milliseconds.
pub fn default_interval_ms(diff: u8) -> u64 {
    match diff {
        0..=1 => 1000,
        2 | 3 => 900,
        4 => 800,
        _ => 700,
    }
}

/// Describe the bead moves on the units rod (showing `u`) to add digit `a`.
pub fn add_move(u: i64, a: i64) -> String {
    let e = u % 5;
    if u + a <= 9 {
        if a >= 5 {
            if a == 5 {
                "lower the heaven bead (+5)".into()
            } else {
                format!("lower the heaven bead and raise {} earth (+5 +{})", a - 5, a - 5)
            }
        } else if e + a <= 4 {
            format!("raise {a} earth (direct)")
        } else {
            format!(
                "5-complement: lower heaven (+5), drop {} earth (−{})",
                5 - a,
                5 - a
            )
        }
    } else {
        let c = 10 - a;
        let remove = if c >= 5 {
            if c == 5 {
                "lift the heaven bead (−5)".to_string()
            } else {
                format!("lift heaven and drop {} earth (−5 −{})", c - 5, c - 5)
            }
        } else if e >= c {
            format!("drop {c} earth")
        } else {
            format!("lift heaven, raise {} earth (−5 +{})", 5 - c, 5 - c)
        };
        format!("10-complement: −{c} on units ({remove}), +1 on tens")
    }
}

pub fn generate(t: usize, diff: u8, rng: &mut impl RngExt) -> Problem {
    let count = 3 + diff as usize;
    let digits = if diff >= 5 { 3 } else { 2 };
    let mut nums: Vec<i64> = Vec::with_capacity(count);
    let mut total = 0i64;
    let mut steps = Vec::new();
    for i in 0..count {
        let n = match t {
            0 => rng.random_range(1..=9),
            _ => n_digits(digits, rng),
        };
        // Subtract only if it keeps the total non-negative, as on a soroban.
        let n = if t == 2 && i > 0 && n <= total && rng.random_bool(0.4) {
            -n
        } else {
            n
        };
        let next = total + n;
        let line = if i == 0 {
            format!("Set {n}")
        } else if t == 0 {
            format!("{total} + {n} = {next}: {}", add_move(total % 10, n))
        } else if n < 0 {
            format!("{total} − {} = {next}", -n)
        } else {
            format!("{total} + {n} = {next}")
        };
        steps.push(line);
        nums.push(n);
        total = next;
    }
    steps.push(format!("Total = {total}"));
    Problem {
        method: Method::Anzan,
        technique: t,
        prompt: format!("Sum of the {count} flashed numbers"),
        answer: Answer::int(total),
        expr: Expr::Sum(nums.clone()),
        steps,
        flash: Some(nums),
    }
}

/// Difficulty 1–2: single digits; 3: multi-digit additions; 4–5: with subtraction.
pub fn generate_random(diff: u8, rng: &mut impl RngExt) -> Problem {
    let t = match diff {
        0..=2 => 0,
        3 => 1,
        _ => 2,
    };
    generate(t, diff, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bead_moves() {
        assert_eq!(add_move(2, 2), "raise 2 earth (direct)");
        assert_eq!(
            add_move(3, 4),
            "5-complement: lower heaven (+5), drop 1 earth (−1)"
        );
        assert_eq!(add_move(1, 7), "lower the heaven bead and raise 2 earth (+5 +2)");
        assert_eq!(
            add_move(7, 8),
            "10-complement: −2 on units (drop 2 earth), +1 on tens"
        );
        assert_eq!(
            add_move(6, 7),
            "10-complement: −3 on units (lift heaven, raise 2 earth (−5 +2)), +1 on tens"
        );
        assert_eq!(
            add_move(9, 1),
            "10-complement: −9 on units (lift heaven and drop 4 earth (−5 −4)), +1 on tens"
        );
    }

    #[test]
    fn subtraction_never_goes_negative() {
        use rand::{SeedableRng, rngs::StdRng};
        let mut rng = StdRng::seed_from_u64(3);
        for _ in 0..2000 {
            let p = generate(2, 5, &mut rng);
            let mut run = 0;
            for n in p.flash.unwrap() {
                run += n;
                assert!(run >= 0);
            }
        }
    }
}
