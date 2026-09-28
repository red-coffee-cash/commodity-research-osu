//! Cross multiplication ("vertically and crosswise"): each answer column is the
//! sum of the digit products whose place values add up to that column.

use rand::RngExt;

use super::{Method, Technique, n_digits};
use crate::problem::{Answer, Expr, Problem, digits_le};

const COLUMN_RULE: &str =
    "Column k sums every product aᵢ×bⱼ with i + j = k (places counted from the right, starting at 0).";
const CARRY_RULE: &str =
    "Go right→left: add the carry, write the units digit, carry the rest (it can be more than 1).";

pub const TECHNIQUES: &[Technique] = &[
    Technique {
        name: "2-digit × 2-digit",
        rule: &[
            "ab × cd: units = b×d; tens = a×d + b×c (the cross); hundreds = a×c.",
            CARRY_RULE,
        ],
    },
    Technique {
        name: "3-digit × 2-digit",
        rule: &[
            "abc × de: units = c×e; tens = b×e + c×d; hundreds = a×e + b×d; thousands = a×d.",
            CARRY_RULE,
        ],
    },
    Technique {
        name: "3-digit × 3-digit",
        rule: &[
            "abc × def: units c·f; tens b·f+c·e; hundreds a·f+b·e+c·d; thousands a·e+b·d; ten-thousands a·d.",
            COLUMN_RULE,
            CARRY_RULE,
        ],
    },
    Technique {
        name: "4-digit × 4-digit",
        rule: &[
            "Same pattern with seven columns; the middle column has four cross products.",
            COLUMN_RULE,
            CARRY_RULE,
        ],
    },
];

const SIZES: [(u32, u32); 4] = [(2, 2), (3, 2), (3, 3), (4, 4)];

fn column_name(k: usize) -> String {
    match k {
        0 => "Units".into(),
        1 => "Tens".into(),
        2 => "Hundreds".into(),
        3 => "Thousands".into(),
        4 => "Ten-thousands".into(),
        5 => "Hundred-thousands".into(),
        6 => "Millions".into(),
        _ => format!("10^{k}"),
    }
}

/// Multiply column by column, returning the result and the worked steps.
pub fn multiply(a: i64, b: i64) -> (i64, Vec<String>) {
    let da = digits_le(a);
    let db = digits_le(b);
    let (p, q) = (da.len(), db.len());
    let mut steps = vec![format!(
        "{a} × {b}: build each column from crosswise digit products."
    )];
    let mut carry = 0i64;
    let mut written: Vec<i64> = Vec::new();
    let cols = p + q - 1;
    let mut leading = 0;
    for k in 0..cols {
        let lo = k.saturating_sub(q - 1);
        let hi = k.min(p - 1);
        // Left digits of `a` first, which is how you read them off the page.
        let pairs: Vec<(i64, i64)> = (lo..=hi).rev().map(|i| (da[i], db[k - i])).collect();
        let prods: Vec<i64> = pairs.iter().map(|(x, y)| x * y).collect();
        let sum: i64 = prods.iter().sum();
        let mut line = format!(
            "{}: {}",
            column_name(k),
            pairs
                .iter()
                .map(|(x, y)| format!("{x}×{y}"))
                .collect::<Vec<_>>()
                .join(" + ")
        );
        if prods.len() > 1 {
            line.push_str(&format!(
                " = {}",
                prods
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" + ")
            ));
        }
        line.push_str(&format!(" = {sum}"));
        let total = sum + carry;
        if carry != 0 {
            line.push_str(&format!(", +carry {carry} = {total}"));
        }
        if k + 1 == cols {
            line.push_str(&format!(" → write {total}"));
            leading = total;
        } else {
            let (w, c) = (total % 10, total / 10);
            line.push_str(&format!(" → write {w}"));
            if c != 0 {
                line.push_str(&format!(", carry {c}"));
            }
            written.push(w);
            carry = c;
        }
        steps.push(line);
    }
    let mut result = leading;
    for w in written.iter().rev() {
        result = result * 10 + w;
    }
    steps.push(format!("{a} × {b} = {result}"));
    (result, steps)
}

pub fn generate(t: usize, _diff: u8, rng: &mut impl RngExt) -> Problem {
    let (da, db) = SIZES[t];
    let a = n_digits(da, rng);
    let b = n_digits(db, rng);
    let (result, steps) = multiply(a, b);
    Problem {
        method: Method::Cross,
        technique: t,
        prompt: format!("{a} × {b}"),
        answer: Answer::int(result),
        expr: Expr::Mul(a, b),
        steps,
        flash: None,
    }
}

/// Difficulty picks the size: 1–2 → 2×2, 3 → 3×2, 4 → 3×3, 5 → 4×4.
pub fn generate_random(diff: u8, rng: &mut impl RngExt) -> Problem {
    let t = match diff {
        0..=2 => 0,
        3 => 1,
        4 => 2,
        _ => 3,
    };
    generate(t, diff, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_two_by_two() {
        for a in 10..100 {
            for b in 10..100 {
                assert_eq!(multiply(a, b).0, a * b);
            }
        }
    }

    #[test]
    fn worked_example() {
        let (r, steps) = multiply(43, 27);
        assert_eq!(r, 1161);
        assert_eq!(steps[1], "Units: 3×7 = 21 → write 1, carry 2");
        assert_eq!(
            steps[2],
            "Tens: 4×7 + 3×2 = 28 + 6 = 34, +carry 2 = 36 → write 6, carry 3"
        );
        assert_eq!(steps[3], "Hundreds: 4×2 = 8, +carry 3 = 11 → write 11");
    }
}
