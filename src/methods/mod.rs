//! The mental-math methods. Each module exposes `TECHNIQUES`, a
//! `generate(technique, difficulty, rng)` that builds a problem with a worked
//! solution, and `generate_random(difficulty, rng)`.

pub mod anzan;
pub mod cross;
pub mod quant;
pub mod trachtenberg;
pub mod vedic;

use rand::RngExt;

use crate::problem::Problem;

pub const MIN_DIFFICULTY: u8 = 1;
pub const MAX_DIFFICULTY: u8 = 5;

/// One concrete trick within a method, with the rule shown in help.
pub struct Technique {
    pub name: &'static str,
    pub rule: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Method {
    Trachtenberg,
    Cross,
    Anzan,
    Vedic,
    Quant,
}

impl Method {
    pub const ALL: [Method; 5] = [
        Method::Trachtenberg,
        Method::Cross,
        Method::Anzan,
        Method::Vedic,
        Method::Quant,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Method::Trachtenberg => "Trachtenberg",
            Method::Cross => "Cross multiplication",
            Method::Anzan => "Anzan (flash)",
            Method::Vedic => "Vedic / base tricks",
            Method::Quant => "Quant staples",
        }
    }

    pub fn techniques(self) -> &'static [Technique] {
        match self {
            Method::Trachtenberg => trachtenberg::TECHNIQUES,
            Method::Cross => cross::TECHNIQUES,
            Method::Anzan => anzan::TECHNIQUES,
            Method::Vedic => vedic::TECHNIQUES,
            Method::Quant => quant::TECHNIQUES,
        }
    }

    pub fn generate_technique(self, t: usize, diff: u8, rng: &mut impl RngExt) -> Problem {
        match self {
            Method::Trachtenberg => trachtenberg::generate(t, diff, rng),
            Method::Cross => cross::generate(t, diff, rng),
            Method::Anzan => anzan::generate(t, diff, rng),
            Method::Vedic => vedic::generate(t, diff, rng),
            Method::Quant => quant::generate(t, diff, rng),
        }
    }

    pub fn generate(self, diff: u8, rng: &mut impl RngExt) -> Problem {
        match self {
            Method::Trachtenberg => trachtenberg::generate_random(diff, rng),
            Method::Cross => cross::generate_random(diff, rng),
            Method::Anzan => anzan::generate_random(diff, rng),
            Method::Vedic => vedic::generate_random(diff, rng),
            Method::Quant => quant::generate_random(diff, rng),
        }
    }
}

/// What a practice session draws problems from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Mixed,
    Only(Method),
}

impl Choice {
    pub const ALL: [Choice; 6] = [
        Choice::Mixed,
        Choice::Only(Method::Trachtenberg),
        Choice::Only(Method::Cross),
        Choice::Only(Method::Anzan),
        Choice::Only(Method::Vedic),
        Choice::Only(Method::Quant),
    ];

    pub fn name(self) -> &'static str {
        match self {
            Choice::Mixed => "Mixed (all methods)",
            Choice::Only(m) => m.name(),
        }
    }

    pub fn generate(self, diff: u8, rng: &mut impl RngExt) -> Problem {
        let method = match self {
            Choice::Mixed => Method::ALL[rng.random_range(0..Method::ALL.len())],
            Choice::Only(m) => m,
        };
        method.generate(diff, rng)
    }
}

/// Pick a random technique index for a method.
pub(crate) fn pick(n: usize, rng: &mut impl RngExt) -> usize {
    rng.random_range(0..n)
}

/// Random number with exactly `digits` digits.
pub(crate) fn n_digits(digits: u32, rng: &mut impl RngExt) -> i64 {
    let lo = 10i64.pow(digits - 1);
    rng.random_range(lo..lo * 10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    /// For every technique and difficulty, the answer produced by the method's
    /// algorithm must match plain arithmetic, and the worked steps must end on
    /// the answer.
    #[test]
    fn every_technique_matches_brute_force() {
        let mut rng = StdRng::seed_from_u64(7);
        for m in Method::ALL {
            for t in 0..m.techniques().len() {
                for diff in MIN_DIFFICULTY..=MAX_DIFFICULTY {
                    for _ in 0..1000 {
                        let p = m.generate_technique(t, diff, &mut rng);
                        let truth = p.expr.eval();
                        assert!(
                            (p.answer.value - truth).abs() < 1e-6,
                            "{} / {}: {} gave {} but truth is {}\n{}",
                            m.name(),
                            p.technique_name(),
                            p.prompt,
                            p.answer.value,
                            truth,
                            p.steps.join("\n")
                        );
                        assert!(p.answer.accepts(truth));
                        let last = p.steps.last().expect("steps");
                        assert!(
                            last.contains(&p.answer.shown),
                            "{}: last step {last:?} lacks {}",
                            p.prompt,
                            p.answer.shown
                        );
                        assert_eq!(p.technique, t);
                    }
                }
            }
        }
    }

    #[test]
    fn random_generation_covers_all_methods() {
        let mut rng = StdRng::seed_from_u64(1);
        for c in Choice::ALL {
            for diff in MIN_DIFFICULTY..=MAX_DIFFICULTY {
                let p = c.generate(diff, &mut rng);
                assert!((p.answer.value - p.expr.eval()).abs() < 1e-6);
            }
        }
    }
}
