//! Problem representation, answer checking and number formatting.

use crate::methods::Method;

/// How close a typed answer must be to count as correct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tol {
    Exact,
    /// Absolute tolerance, e.g. answers rounded to 2 decimal places.
    Abs(f64),
    /// Relative tolerance, used for estimation drills.
    Rel(f64),
}

#[derive(Clone, Debug)]
pub struct Answer {
    pub value: f64,
    pub tol: Tol,
    /// Canonical text of the answer, as shown to the user.
    pub shown: String,
}

impl Answer {
    pub fn int(v: i64) -> Self {
        Self {
            value: v as f64,
            tol: Tol::Exact,
            shown: v.to_string(),
        }
    }

    /// An exact decimal answer (terminating), shown with up to 4 decimals.
    pub fn exact(v: f64) -> Self {
        Self {
            value: v,
            tol: Tol::Exact,
            shown: fmt_num(v, 4),
        }
    }

    /// An answer the user should give rounded to `dp` decimal places.
    pub fn rounded(v: f64, dp: i32) -> Self {
        Self {
            value: v,
            tol: Tol::Abs(10f64.powi(-dp)),
            shown: fmt_num(v, dp as usize),
        }
    }

    /// An estimate that must land within `rel` (e.g. 0.02 = 2%) of `v`.
    pub fn approx(v: f64, rel: f64) -> Self {
        Self {
            value: v,
            tol: Tol::Rel(rel),
            shown: fmt_num(v, 2),
        }
    }

    pub fn accepts(&self, x: f64) -> bool {
        let diff = (x - self.value).abs();
        match self.tol {
            Tol::Exact => diff < 1e-6,
            Tol::Abs(t) => diff <= t + 1e-9,
            Tol::Rel(r) => diff <= r * self.value.abs() + 1e-9,
        }
    }

    /// Short note on what counts as correct, if it's not an exact match.
    pub fn tolerance_note(&self) -> Option<String> {
        match self.tol {
            Tol::Exact => None,
            Tol::Abs(t) => Some(format!("within ±{}", fmt_num(t, 4))),
            Tol::Rel(r) => Some(format!("within {}%", fmt_num(r * 100.0, 2))),
        }
    }
}

/// The plain arithmetic a problem stands for. Tests evaluate this by brute
/// force and compare with the answer the method's algorithm produced.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Debug)]
pub enum Expr {
    Mul(i64, i64),
    Sum(Vec<i64>),
    /// Numerator / denominator.
    Ratio(i64, i64),
    Pow2(u32),
    /// Win `win` with probability num/den, otherwise lose `lose`.
    Ev {
        num: i64,
        den: i64,
        win: i64,
        lose: i64,
    },
}

impl Expr {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn eval(&self) -> f64 {
        match self {
            Expr::Mul(a, b) => (a * b) as f64,
            Expr::Sum(v) => v.iter().sum::<i64>() as f64,
            Expr::Ratio(n, d) => *n as f64 / *d as f64,
            Expr::Pow2(k) => (1u64 << k) as f64,
            Expr::Ev { num, den, win, lose } => (num * win - (den - num) * lose) as f64 / *den as f64,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Problem {
    pub method: Method,
    /// Index into `method.techniques()`.
    pub technique: usize,
    pub prompt: String,
    pub answer: Answer,
    #[cfg_attr(not(test), allow(dead_code))]
    pub expr: Expr,
    /// Worked solution using the technique; the last line holds the answer.
    pub steps: Vec<String>,
    /// For flash anzan: the numbers to flash before the prompt appears.
    pub flash: Option<Vec<i64>>,
}

impl Problem {
    /// The prompt, or for flashed problems the sequence itself ("7 + 8 − 3").
    pub fn short_prompt(&self) -> String {
        let Some(nums) = &self.flash else {
            return self.prompt.clone();
        };
        let mut s = String::new();
        for (i, n) in nums.iter().enumerate() {
            match (i, *n < 0) {
                (0, _) => s.push_str(&n.to_string()),
                (_, true) => s.push_str(&format!(" − {}", -n)),
                (_, false) => s.push_str(&format!(" + {n}")),
            }
        }
        s
    }

    pub fn technique_name(&self) -> &'static str {
        self.method.techniques()[self.technique].name
    }
}

/// Format a number with at most `max_dp` decimals, trimming trailing zeros.
pub fn fmt_num(v: f64, max_dp: usize) -> String {
    let s = format!("{:.*}", max_dp, v);
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    if s == "-0" { "0".to_string() } else { s }
}

/// Parse a typed answer. Accepts integers, decimals, fractions ("3/8"), and
/// ignores `%`, `$`, `,` and spaces.
pub fn parse_input(s: &str) -> Option<f64> {
    let cleaned: String = s
        .chars()
        .filter(|c| !matches!(c, '%' | '$' | ',' | ' ' | '_'))
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    if let Some((n, d)) = cleaned.split_once('/') {
        let n: f64 = n.parse().ok()?;
        let d: f64 = d.parse().ok()?;
        if d == 0.0 {
            return None;
        }
        return Some(n / d);
    }
    let v: f64 = cleaned.parse().ok()?;
    v.is_finite().then_some(v)
}

/// Digits of a non-negative number, least significant first. 0 gives [0].
pub fn digits_le(n: i64) -> Vec<i64> {
    let mut n = n.abs();
    let mut out = vec![];
    loop {
        out.push(n % 10);
        n /= 10;
        if n == 0 {
            return out;
        }
    }
}

/// Join signed terms as "a + b − c".
pub fn join_terms(terms: &[(String, i64)]) -> String {
    let mut s = String::new();
    for (i, (label, _)) in terms.iter().enumerate() {
        match (i, label.strip_prefix('−')) {
            (0, _) => s.push_str(label),
            (_, Some(rest)) => {
                s.push_str(" − ");
                s.push_str(rest);
            }
            (_, None) => {
                s.push_str(" + ");
                s.push_str(label);
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_formats() {
        assert_eq!(parse_input("1,234"), Some(1234.0));
        assert_eq!(parse_input("37.5%"), Some(37.5));
        assert_eq!(parse_input("3/8"), Some(0.375));
        assert_eq!(parse_input("-12"), Some(-12.0));
        assert_eq!(parse_input("$4.50"), Some(4.5));
        assert_eq!(parse_input(""), None);
        assert_eq!(parse_input("1/0"), None);
        assert_eq!(parse_input("abc"), None);
    }

    #[test]
    fn tolerances() {
        assert!(Answer::int(42).accepts(42.0));
        assert!(!Answer::int(42).accepts(41.0));
        let r = Answer::rounded(100.0 / 7.0, 2);
        assert_eq!(r.shown, "14.29");
        assert!(r.accepts(14.29) && r.accepts(14.28) && !r.accepts(14.3));
        let a = Answer::approx(1000.0, 0.02);
        assert!(a.accepts(1019.0) && a.accepts(981.0) && !a.accepts(1021.0));
    }

    #[test]
    fn formatting() {
        assert_eq!(fmt_num(57.80000001, 4), "57.8");
        assert_eq!(fmt_num(3.0, 4), "3");
        assert_eq!(fmt_num(-0.00001, 2), "0");
        assert_eq!(digits_le(4072), vec![2, 7, 0, 4]);
        let t = [
            ("9−3".to_string(), 6),
            ("−1".to_string(), -1),
            ("4".to_string(), 4),
        ];
        assert_eq!(join_terms(&t), "9−3 − 1 + 4");
    }
}
