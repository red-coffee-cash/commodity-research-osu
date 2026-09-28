//! Trachtenberg single-multiplier rules. Every rule works digit by digit,
//! right to left, on the multiplicand written with a leading zero. The
//! "neighbour" of a digit is the digit immediately to its right.

use rand::RngExt;

use super::{Method, Technique, n_digits, pick};
use crate::problem::{Answer, Expr, Problem, digits_le, join_terms};

const COMMON: &str =
    "Write the number with a leading 0; go right→left; neighbour = digit to the right (0 past the end).";

pub const TECHNIQUES: &[Technique] = &[
    Technique {
        name: "×11",
        rule: &[COMMON, "Each answer digit = digit + neighbour. Carry as usual."],
    },
    Technique {
        name: "×12",
        rule: &[COMMON, "Each answer digit = 2 × digit + neighbour."],
    },
    Technique {
        name: "×5",
        rule: &[
            COMMON,
            "Each answer digit = half the neighbour (drop fractions), +5 if the digit is odd.",
        ],
    },
    Technique {
        name: "×6",
        rule: &[
            COMMON,
            "Each answer digit = digit + half the neighbour, +5 if the digit is odd.",
        ],
    },
    Technique {
        name: "×7",
        rule: &[
            COMMON,
            "Each answer digit = 2 × digit + half the neighbour, +5 if the digit is odd.",
        ],
    },
    Technique {
        name: "×9",
        rule: &[
            COMMON,
            "Rightmost digit: 10 − digit.",
            "Middle digits: (9 − digit) + neighbour.",
            "Leading 0: leftmost digit of the number − 1.",
        ],
    },
    Technique {
        name: "×8",
        rule: &[
            COMMON,
            "Rightmost digit: 2 × (10 − digit).",
            "Middle digits: 2 × (9 − digit) + neighbour.",
            "Leading 0: leftmost digit of the number − 2.",
        ],
    },
    Technique {
        name: "×4",
        rule: &[
            COMMON,
            "Rightmost digit: (10 − digit), +5 if the digit is odd.",
            "Middle digits: (9 − digit) + half the neighbour, +5 if the digit is odd.",
            "Leading 0: half the leftmost digit − 1.",
        ],
    },
    Technique {
        name: "×3",
        rule: &[
            COMMON,
            "Rightmost digit: 2 × (10 − digit), +5 if the digit is odd.",
            "Middle digits: 2 × (9 − digit) + half the neighbour, +5 if the digit is odd.",
            "Leading 0: half the leftmost digit − 2.",
        ],
    },
];

const MULTIPLIERS: [i64; 9] = [11, 12, 5, 6, 7, 9, 8, 4, 3];

#[derive(Clone, Copy, PartialEq)]
enum Pos {
    Right,
    Middle,
    Leading,
}

/// The terms the rule adds up for one answer position.
fn terms(m: i64, d: i64, r: i64, pos: Pos) -> Vec<(String, i64)> {
    let mut t: Vec<(String, i64)> = Vec::new();
    let half = || (format!("½·{r}"), r / 2);
    let odd5 = |t: &mut Vec<(String, i64)>| {
        if d % 2 == 1 {
            t.push(("5 (odd)".into(), 5));
        }
    };
    match m {
        11 => {
            t.push((d.to_string(), d));
            t.push((r.to_string(), r));
        }
        12 => {
            t.push((format!("2×{d}"), 2 * d));
            t.push((r.to_string(), r));
        }
        5 => {
            t.push(half());
            odd5(&mut t);
        }
        6 => {
            t.push((d.to_string(), d));
            t.push(half());
            odd5(&mut t);
        }
        7 => {
            t.push((format!("2×{d}"), 2 * d));
            t.push(half());
            odd5(&mut t);
        }
        9 => match pos {
            Pos::Right => t.push((format!("(10−{d})"), 10 - d)),
            Pos::Middle => {
                t.push((format!("(9−{d})"), 9 - d));
                t.push((r.to_string(), r));
            }
            Pos::Leading => {
                t.push((r.to_string(), r));
                t.push(("−1".into(), -1));
            }
        },
        8 => match pos {
            Pos::Right => t.push((format!("2×(10−{d})"), 2 * (10 - d))),
            Pos::Middle => {
                t.push((format!("2×(9−{d})"), 2 * (9 - d)));
                t.push((r.to_string(), r));
            }
            Pos::Leading => {
                t.push((r.to_string(), r));
                t.push(("−2".into(), -2));
            }
        },
        4 => match pos {
            Pos::Right => {
                t.push((format!("(10−{d})"), 10 - d));
                odd5(&mut t);
            }
            Pos::Middle => {
                t.push((format!("(9−{d})"), 9 - d));
                t.push(half());
                odd5(&mut t);
            }
            Pos::Leading => {
                t.push(half());
                t.push(("−1".into(), -1));
            }
        },
        3 => match pos {
            Pos::Right => {
                t.push((format!("2×(10−{d})"), 2 * (10 - d)));
                odd5(&mut t);
            }
            Pos::Middle => {
                t.push((format!("2×(9−{d})"), 2 * (9 - d)));
                t.push(half());
                odd5(&mut t);
            }
            Pos::Leading => {
                t.push(half());
                t.push(("−2".into(), -2));
            }
        },
        _ => unreachable!("unsupported multiplier {m}"),
    }
    t
}

/// Run the Trachtenberg rule for `n × m`, returning the result and the steps.
pub fn multiply(n: i64, m: i64) -> (i64, Vec<String>) {
    let ds = digits_le(n);
    let len = ds.len();
    let mut steps = vec![format!(
        "Write 0{n}; work right→left. Neighbour = the digit to the right."
    )];
    let mut carry = 0i64;
    let mut written: Vec<i64> = Vec::new(); // least significant first
    let mut leading = 0i64;
    for i in 0..=len {
        let d = if i < len { ds[i] } else { 0 };
        let r = if i == 0 { 0 } else { ds[i - 1] };
        let pos = if i == len {
            Pos::Leading
        } else if i == 0 {
            Pos::Right
        } else {
            Pos::Middle
        };
        let t = terms(m, d, r, pos);
        let value: i64 = t.iter().map(|x| x.1).sum();
        let total = value + carry;
        let carry_txt = if carry != 0 {
            format!(", +carry {carry} = {total}")
        } else {
            String::new()
        };
        let what = match pos {
            Pos::Leading => format!("Leading 0 (neighbour {r})"),
            Pos::Right => format!("Digit {d}"),
            Pos::Middle => format!("Digit {d} (neighbour {r})"),
        };
        if pos == Pos::Leading {
            leading = total;
            let out = if total == 0 {
                "nothing to write".to_string()
            } else {
                format!("write {total}")
            };
            steps.push(format!("{what}: {} = {value}{carry_txt} → {out}", join_terms(&t)));
        } else {
            let w = total.rem_euclid(10);
            let c = total.div_euclid(10);
            let c_txt = if c != 0 {
                format!(", carry {c}")
            } else {
                String::new()
            };
            steps.push(format!(
                "{what}: {} = {value}{carry_txt} → write {w}{c_txt}",
                join_terms(&t)
            ));
            written.push(w);
            carry = c;
        }
    }
    let mut result = leading;
    for w in written.iter().rev() {
        result = result * 10 + w;
    }
    steps.push(format!(
        "Read the written digits left→right: {n} × {m} = {result}"
    ));
    (result, steps)
}

pub fn generate(t: usize, diff: u8, rng: &mut impl RngExt) -> Problem {
    let m = MULTIPLIERS[t];
    let n = n_digits(diff as u32 + 1, rng);
    let (result, steps) = multiply(n, m);
    Problem {
        method: Method::Trachtenberg,
        technique: t,
        prompt: format!("{n} × {m}"),
        answer: Answer::int(result),
        expr: Expr::Mul(n, m),
        steps,
        flash: None,
    }
}

pub fn generate_random(diff: u8, rng: &mut impl RngExt) -> Problem {
    generate(pick(TECHNIQUES.len(), rng), diff, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_small_numbers() {
        for m in MULTIPLIERS {
            for n in 1..=5000 {
                assert_eq!(multiply(n, m).0, n * m, "{n} × {m}");
            }
        }
    }

    #[test]
    fn worked_example_times_7() {
        // 342 × 7: digit 2 → 4; digit 4 (nbr 2) → 8+1=9; digit 3 (nbr 4) → 6+2+5=13;
        // leading (nbr 3) → 1 + carry 1 = 2. Answer 2394.
        let (r, steps) = multiply(342, 7);
        assert_eq!(r, 2394);
        assert_eq!(steps[1], "Digit 2: 2×2 + ½·0 = 4 → write 4");
        assert_eq!(
            steps[3],
            "Digit 3 (neighbour 4): 2×3 + ½·4 + 5 (odd) = 13 → write 3, carry 1"
        );
    }
}
