//! A practice session: problem flow, timing, flash sequencing and scoring.
//! All time-dependent methods take `now` so they can be tested with a fake clock.

use std::time::{Duration, Instant};

use rand::RngExt;

use crate::methods::{Choice, Method};
use crate::problem::{Problem, parse_input};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Unlimited,
    Timed(Duration),
}

#[derive(Clone, Debug)]
pub struct Record {
    pub method: Method,
    pub prompt: String,
    pub expected: String,
    /// None when skipped.
    pub given: Option<String>,
    pub correct: bool,
    pub time: Duration,
}

/// Result of the previous problem, shown under the current one.
#[derive(Clone, Debug)]
pub struct Feedback {
    pub correct: bool,
    pub given: Option<String>,
    pub problem: Problem,
    pub time: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashView {
    Show(usize),
    /// Short blank between numbers so repeats are visible.
    Gap(usize),
    Done,
}

/// Which flashed number (if any) is visible `elapsed` after the start.
pub fn flash_view(elapsed: Duration, interval: Duration, n: usize) -> FlashView {
    let slot = interval.as_millis().max(1);
    let e = elapsed.as_millis();
    let idx = (e / slot) as usize;
    if idx >= n {
        return FlashView::Done;
    }
    let gap = (slot / 5).min(150);
    if e % slot >= slot - gap {
        FlashView::Gap(idx)
    } else {
        FlashView::Show(idx)
    }
}

pub fn timed_key(choice: Choice, difficulty: u8, secs: u64) -> String {
    format!("{} · level {difficulty} · {secs}s", choice.name())
}

pub const MIN_FLASH_MS: u64 = 100;
pub const MAX_FLASH_MS: u64 = 3000;

pub struct Session {
    pub mode: Mode,
    pub choice: Choice,
    pub difficulty: u8,
    pub started: Instant,
    pub problem: Problem,
    pub shown_at: Instant,
    pub input: String,
    pub records: Vec<Record>,
    pub streak: u32,
    pub best_streak: u32,
    pub feedback: Option<Feedback>,
    pub error: Option<String>,
    pub flash_interval: Duration,
    pub finished_at: Option<Instant>,
}

impl Session {
    pub fn new(
        mode: Mode,
        choice: Choice,
        difficulty: u8,
        flash_interval: Duration,
        rng: &mut impl RngExt,
        now: Instant,
    ) -> Self {
        Session {
            mode,
            choice,
            difficulty,
            started: now,
            problem: choice.generate(difficulty, rng),
            shown_at: now,
            input: String::new(),
            records: Vec::new(),
            streak: 0,
            best_streak: 0,
            feedback: None,
            error: None,
            flash_interval,
            finished_at: None,
        }
    }

    pub fn flash_total(&self) -> Duration {
        let n = self.problem.flash.as_ref().map_or(0, |f| f.len());
        self.flash_interval * n as u32
    }

    /// The current flash state, or None if this problem isn't flashed.
    pub fn flash(&self, now: Instant) -> Option<FlashView> {
        let nums = self.problem.flash.as_ref()?;
        Some(flash_view(
            now.saturating_duration_since(self.shown_at),
            self.flash_interval,
            nums.len(),
        ))
    }

    pub fn input_ready(&self, now: Instant) -> bool {
        !matches!(self.flash(now), Some(FlashView::Show(_) | FlashView::Gap(_)))
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        self.finished_at
            .unwrap_or(now)
            .saturating_duration_since(self.started)
    }

    pub fn remaining(&self, now: Instant) -> Option<Duration> {
        match self.mode {
            Mode::Timed(d) => Some(d.saturating_sub(self.elapsed(now))),
            Mode::Unlimited => None,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }

    /// Ends a timed session once its clock runs out.
    pub fn tick(&mut self, now: Instant) {
        if let (Mode::Timed(d), None) = (self.mode, self.finished_at)
            && now.saturating_duration_since(self.started) >= d
        {
            self.finished_at = Some(self.started + d);
        }
    }

    pub fn finish(&mut self, now: Instant) {
        if self.finished_at.is_none() {
            self.finished_at = Some(now);
        }
    }

    /// Time spent on the current problem since it became answerable.
    pub fn solve_time(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.shown_at + self.flash_total())
    }

    pub fn push_char(&mut self, c: char, now: Instant) {
        if self.input_ready(now) && self.input.len() < 24 {
            self.input.push(c);
            self.error = None;
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        self.error = None;
    }

    /// Check the typed answer. Returns Some(correct) if it was graded.
    pub fn submit(&mut self, now: Instant, rng: &mut impl RngExt) -> Option<bool> {
        if self.is_finished() || !self.input_ready(now) || self.input.trim().is_empty() {
            return None;
        }
        let Some(value) = parse_input(&self.input) else {
            self.error = Some(format!("Couldn't read {:?} as a number", self.input));
            return None;
        };
        let correct = self.problem.answer.accepts(value);
        let given = std::mem::take(&mut self.input);
        self.record(Some(given), correct, now, rng);
        Some(correct)
    }

    /// Give up on the current problem and reveal the worked solution.
    pub fn skip(&mut self, now: Instant, rng: &mut impl RngExt) {
        if self.is_finished() || !self.input_ready(now) {
            return;
        }
        self.input.clear();
        self.record(None, false, now, rng);
    }

    /// Show the flash sequence again (the clock keeps running).
    pub fn replay_flash(&mut self, now: Instant) {
        if self.problem.flash.is_some() {
            self.shown_at = now;
            self.input.clear();
        }
    }

    pub fn adjust_flash(&mut self, delta_ms: i64) {
        let ms = (self.flash_interval.as_millis() as i64 + delta_ms)
            .clamp(MIN_FLASH_MS as i64, MAX_FLASH_MS as i64);
        self.flash_interval = Duration::from_millis(ms as u64);
    }

    fn record(&mut self, given: Option<String>, correct: bool, now: Instant, rng: &mut impl RngExt) {
        let time = self.solve_time(now);
        if correct {
            self.streak += 1;
            self.best_streak = self.best_streak.max(self.streak);
        } else {
            self.streak = 0;
        }
        self.records.push(Record {
            method: self.problem.method,
            prompt: self.problem.short_prompt(),
            expected: self.problem.answer.shown.clone(),
            given: given.clone(),
            correct,
            time,
        });
        let next = self.choice.generate(self.difficulty, rng);
        let problem = std::mem::replace(&mut self.problem, next);
        self.feedback = Some(Feedback {
            correct,
            given,
            problem,
            time,
        });
        self.shown_at = now;
        self.error = None;
    }

    pub fn correct(&self) -> usize {
        self.records.iter().filter(|r| r.correct).count()
    }

    pub fn accuracy(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.correct() as f64 / self.records.len() as f64
        }
    }

    /// Average time over correctly answered problems.
    pub fn avg_correct_time(&self) -> Option<Duration> {
        let times: Vec<Duration> = self
            .records
            .iter()
            .filter(|r| r.correct)
            .map(|r| r.time)
            .collect();
        (!times.is_empty()).then(|| times.iter().sum::<Duration>() / times.len() as u32)
    }

    /// Key for storing the best score of this session's timed configuration.
    pub fn timed_key(&self) -> Option<String> {
        match self.mode {
            Mode::Timed(d) => Some(timed_key(self.choice, self.difficulty, d.as_secs())),
            Mode::Unlimited => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::Method;
    use rand::{SeedableRng, rngs::StdRng};

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn flash_sequence() {
        let i = ms(1000);
        assert_eq!(flash_view(ms(0), i, 3), FlashView::Show(0));
        assert_eq!(flash_view(ms(849), i, 3), FlashView::Show(0));
        assert_eq!(flash_view(ms(850), i, 3), FlashView::Gap(0));
        assert_eq!(flash_view(ms(1000), i, 3), FlashView::Show(1));
        assert_eq!(flash_view(ms(2999), i, 3), FlashView::Gap(2));
        assert_eq!(flash_view(ms(3000), i, 3), FlashView::Done);
        // Fast flashes keep a gap of a fifth of the slot.
        assert_eq!(flash_view(ms(80), ms(100), 2), FlashView::Gap(0));
    }

    #[test]
    fn timed_session_ends_on_clock() {
        let mut rng = StdRng::seed_from_u64(1);
        let t0 = Instant::now();
        let mut s = Session::new(
            Mode::Timed(Duration::from_secs(30)),
            Choice::Only(Method::Vedic),
            1,
            ms(500),
            &mut rng,
            t0,
        );
        s.tick(t0 + Duration::from_secs(29));
        assert!(!s.is_finished());
        assert_eq!(
            s.remaining(t0 + Duration::from_secs(29)),
            Some(Duration::from_secs(1))
        );
        s.tick(t0 + Duration::from_secs(31));
        assert!(s.is_finished());
        // The clock stops at the limit even if checked later.
        assert_eq!(s.elapsed(t0 + Duration::from_secs(99)), Duration::from_secs(30));
        s.input = "1".into();
        assert_eq!(s.submit(t0 + Duration::from_secs(32), &mut rng), None);
    }

    #[test]
    fn grading_streaks_and_timing() {
        let mut rng = StdRng::seed_from_u64(2);
        let t0 = Instant::now();
        let mut s = Session::new(
            Mode::Unlimited,
            Choice::Only(Method::Trachtenberg),
            2,
            ms(500),
            &mut rng,
            t0,
        );
        let ans = s.problem.answer.shown.clone();
        s.input = ans;
        assert_eq!(s.submit(t0 + ms(2500), &mut rng), Some(true));
        assert_eq!(s.records[0].time, ms(2500));
        assert_eq!(s.streak, 1);

        s.input = "abc".into();
        assert_eq!(s.submit(t0 + ms(3000), &mut rng), None);
        assert!(s.error.is_some());

        s.input = "-1".into();
        assert_eq!(s.submit(t0 + ms(4000), &mut rng), Some(false));
        assert_eq!((s.streak, s.best_streak), (0, 1));
        s.skip(t0 + ms(5000), &mut rng);
        assert_eq!(s.records.len(), 3);
        assert_eq!(s.correct(), 1);
        assert_eq!(s.avg_correct_time(), Some(ms(2500)));
        assert!(s.records[2].given.is_none());
    }

    #[test]
    fn flash_blocks_input_and_timing_starts_after() {
        let mut rng = StdRng::seed_from_u64(3);
        let t0 = Instant::now();
        let mut s = Session::new(
            Mode::Unlimited,
            Choice::Only(Method::Anzan),
            1,
            ms(500),
            &mut rng,
            t0,
        );
        let n = s.problem.flash.as_ref().unwrap().len() as u64;
        s.push_char('1', t0 + ms(100));
        assert!(s.input.is_empty());
        let done = t0 + ms(500 * n);
        assert!(s.input_ready(done));
        s.input = s.problem.answer.shown.clone();
        assert_eq!(s.submit(done + ms(1200), &mut rng), Some(true));
        assert_eq!(s.records[0].time, ms(1200));
    }
}
