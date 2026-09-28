//! Application state and key handling.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use rand::rngs::StdRng;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::methods::{Choice, MAX_DIFFICULTY, MIN_DIFFICULTY, Method, anzan};
use crate::problem::Problem;
use crate::session::{Mode, Session};
use crate::stats::{self, Stats};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Setup,
    Session,
    Summary,
    Help,
    Soroban,
    Stats,
}

pub const MENU: [&str; 6] = [
    "Unlimited practice",
    "Timed practice",
    "Methods & help",
    "Soroban visualizer",
    "Stats",
    "Quit",
];

pub const DURATIONS: [u64; 5] = [30, 60, 120, 300, 600];

pub struct Setup {
    pub timed: bool,
    pub choice: usize,
    pub difficulty: u8,
    pub duration: usize,
}

pub struct Summary {
    pub session: Session,
    pub new_best: bool,
    pub previous_best: Option<u32>,
}

pub struct HelpState {
    /// 0 = overview, 1.. = Method::ALL[page - 1].
    pub page: usize,
    pub scroll: u16,
    pub difficulty: u8,
    pub examples: Vec<Problem>,
}

pub struct SorobanState {
    pub value: u64,
}

pub struct App {
    pub screen: Screen,
    pub menu: usize,
    pub setup: Setup,
    pub session: Option<Session>,
    pub summary: Option<Summary>,
    pub help: HelpState,
    pub soroban: SorobanState,
    /// Show the current technique's rule over a session.
    pub rule_overlay: bool,
    /// Draw flashed numbers on a soroban as well as in digits.
    pub flash_soroban: bool,
    /// Show the worked solution after correct answers too.
    pub always_steps: bool,
    pub stats: Stats,
    pub stats_path: Option<PathBuf>,
    /// A message shown in the footer (save errors, etc).
    pub status: Option<String>,
    pub rng: StdRng,
    pub quit: bool,
}

impl App {
    pub fn new() -> Self {
        let stats_path = stats::default_path();
        let (stats, status) = match &stats_path {
            Some(p) => Stats::load(p),
            None => (
                Stats::default(),
                Some("No data directory; stats won't be saved".into()),
            ),
        };
        let mut app = App {
            screen: Screen::Menu,
            menu: 0,
            setup: Setup {
                timed: false,
                choice: 0,
                difficulty: 2,
                duration: 1,
            },
            session: None,
            summary: None,
            help: HelpState {
                page: 0,
                scroll: 0,
                difficulty: 2,
                examples: vec![],
            },
            soroban: SorobanState { value: 1234 },
            rule_overlay: false,
            flash_soroban: false,
            always_steps: false,
            stats,
            stats_path,
            status,
            rng: rand::make_rng(),
            quit: false,
        };
        app.refresh_examples();
        app
    }

    pub fn tick(&mut self, now: Instant) {
        if let Some(s) = &mut self.session {
            s.tick(now);
            if s.is_finished() {
                self.end_session(now);
            }
        }
    }

    pub fn on_key(&mut self, key: KeyEvent, now: Instant) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.end_session(now);
            self.quit = true;
            return;
        }
        match self.screen {
            Screen::Menu => self.menu_key(key.code),
            Screen::Setup => self.setup_key(key.code, now),
            Screen::Session => self.session_key(key.code, now),
            Screen::Summary => self.summary_key(key.code, now),
            Screen::Help => self.help_key(key.code),
            Screen::Soroban => self.soroban_key(key.code),
            Screen::Stats => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter) {
                    self.screen = Screen::Menu;
                }
            }
        }
    }

    fn menu_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.menu = (self.menu + MENU.len() - 1) % MENU.len(),
            KeyCode::Down | KeyCode::Char('j') => self.menu = (self.menu + 1) % MENU.len(),
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Enter => match self.menu {
                0 | 1 => {
                    self.setup.timed = self.menu == 1;
                    self.screen = Screen::Setup;
                }
                2 => self.screen = Screen::Help,
                3 => self.screen = Screen::Soroban,
                4 => self.screen = Screen::Stats,
                _ => self.quit = true,
            },
            _ => {}
        }
    }

    fn setup_key(&mut self, code: KeyCode, now: Instant) {
        let n = Choice::ALL.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.setup.choice = (self.setup.choice + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.setup.choice = (self.setup.choice + 1) % n,
            KeyCode::Left | KeyCode::Char('h') => {
                self.setup.difficulty = (self.setup.difficulty - 1).max(MIN_DIFFICULTY)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.setup.difficulty = (self.setup.difficulty + 1).min(MAX_DIFFICULTY)
            }
            KeyCode::Char('t') => self.setup.duration = (self.setup.duration + 1) % DURATIONS.len(),
            KeyCode::Tab => self.setup.timed = !self.setup.timed,
            KeyCode::Enter => self.start_session(now),
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Menu,
            _ => {}
        }
    }

    fn start_session(&mut self, now: Instant) {
        let mode = if self.setup.timed {
            Mode::Timed(Duration::from_secs(DURATIONS[self.setup.duration]))
        } else {
            Mode::Unlimited
        };
        let diff = self.setup.difficulty;
        let interval = Duration::from_millis(anzan::default_interval_ms(diff));
        let choice = Choice::ALL[self.setup.choice];
        self.session = Some(Session::new(mode, choice, diff, interval, &mut self.rng, now));
        self.rule_overlay = false;
        self.screen = Screen::Session;
    }

    fn session_key(&mut self, code: KeyCode, now: Instant) {
        let Some(s) = &mut self.session else {
            self.screen = Screen::Menu;
            return;
        };
        if self.rule_overlay {
            // Any key closes the overlay; Esc shouldn't also end the session.
            self.rule_overlay = false;
            if matches!(code, KeyCode::Esc | KeyCode::Char('?')) {
                return;
            }
        }
        match code {
            KeyCode::Char(c @ ('0'..='9' | '.' | '-' | '/')) => s.push_char(c, now),
            KeyCode::Backspace => s.backspace(),
            KeyCode::Enter => {
                s.submit(now, &mut self.rng);
            }
            KeyCode::Tab => s.skip(now, &mut self.rng),
            KeyCode::Char('?') => self.rule_overlay = true,
            KeyCode::Char('[') => s.adjust_flash(100),
            KeyCode::Char(']') => s.adjust_flash(-100),
            KeyCode::Char('r') => s.replay_flash(now),
            KeyCode::Char('s') => self.flash_soroban = !self.flash_soroban,
            KeyCode::Char('w') => self.always_steps = !self.always_steps,
            KeyCode::Esc => self.end_session(now),
            _ => {}
        }
    }

    /// Finish the active session, fold it into stats, save, show the summary.
    fn end_session(&mut self, now: Instant) {
        let Some(mut s) = self.session.take() else { return };
        s.finish(now);
        self.stats.record(&s.records);
        let mut new_best = false;
        let mut previous_best = None;
        if let Some(key) = s.timed_key() {
            previous_best = self.stats.timed_best.get(&key).copied();
            new_best = self.stats.submit_timed(key, s.correct() as u32);
        }
        if !s.records.is_empty() || new_best {
            self.save_stats();
        }
        self.summary = Some(Summary {
            session: s,
            new_best,
            previous_best,
        });
        self.screen = Screen::Summary;
    }

    fn save_stats(&mut self) {
        if let Some(p) = &self.stats_path
            && let Err(e) = self.stats.save(p)
        {
            self.status = Some(format!("Couldn't save stats: {e:#}"));
        }
    }

    fn summary_key(&mut self, code: KeyCode, now: Instant) {
        match code {
            KeyCode::Char('r') => self.start_session(now),
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Menu,
            _ => {}
        }
    }

    pub fn help_method(&self) -> Option<Method> {
        self.help.page.checked_sub(1).map(|i| Method::ALL[i])
    }

    fn refresh_examples(&mut self) {
        self.help.examples = match self.help_method() {
            Some(m) => (0..m.techniques().len())
                .map(|t| m.generate_technique(t, self.help.difficulty, &mut self.rng))
                .collect(),
            None => vec![],
        };
    }

    fn help_key(&mut self, code: KeyCode) {
        let pages = Method::ALL.len() + 1;
        match code {
            KeyCode::Up => {
                self.help.page = (self.help.page + pages - 1) % pages;
                self.help.scroll = 0;
                self.refresh_examples();
            }
            KeyCode::Down => {
                self.help.page = (self.help.page + 1) % pages;
                self.help.scroll = 0;
                self.refresh_examples();
            }
            KeyCode::Char('j') => self.help.scroll = self.help.scroll.saturating_add(1),
            KeyCode::Char('k') => self.help.scroll = self.help.scroll.saturating_sub(1),
            KeyCode::PageDown | KeyCode::Char(' ') => self.help.scroll = self.help.scroll.saturating_add(10),
            KeyCode::PageUp => self.help.scroll = self.help.scroll.saturating_sub(10),
            KeyCode::Left => {
                self.help.difficulty = (self.help.difficulty - 1).max(MIN_DIFFICULTY);
                self.refresh_examples();
            }
            KeyCode::Right => {
                self.help.difficulty = (self.help.difficulty + 1).min(MAX_DIFFICULTY);
                self.refresh_examples();
            }
            KeyCode::Char('n') => self.refresh_examples(),
            KeyCode::Char('p') => {
                if let Some(m) = self.help_method() {
                    self.setup.choice = Choice::ALL
                        .iter()
                        .position(|c| *c == Choice::Only(m))
                        .unwrap_or(0);
                    self.setup.timed = false;
                    self.screen = Screen::Setup;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Menu,
            _ => {}
        }
    }

    fn soroban_key(&mut self, code: KeyCode) {
        const MAX: u64 = 999_999_999_999;
        let v = &mut self.soroban.value;
        match code {
            KeyCode::Char(c @ '0'..='9') => {
                let next = *v * 10 + c.to_digit(10).unwrap() as u64;
                if next <= MAX {
                    *v = next;
                }
            }
            KeyCode::Backspace => *v /= 10,
            KeyCode::Up => *v = (*v + 1).min(MAX),
            KeyCode::Down => *v = v.saturating_sub(1),
            KeyCode::PageUp => *v = (*v + 10).min(MAX),
            KeyCode::PageDown => *v = v.saturating_sub(10),
            KeyCode::Char('x') => *v = 0,
            KeyCode::Char('r') => {
                use rand::RngExt;
                *v = self.rng.random_range(0..100_000);
            }
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Menu,
            _ => {}
        }
    }
}
