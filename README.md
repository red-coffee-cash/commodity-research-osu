# mentalmath

A terminal app for drilling mental math for quant interviews. It covers the
Trachtenberg system, cross multiplication, flash anzan (with a soroban
visualizer), Vedic/base tricks and quant staples (fractions, percentages,
estimation, powers of two, expected value).

```
cargo run --release
```

## Modes

- **Unlimited practice**: problems keep coming. The header shows your streak,
  accuracy and average time. Esc ends the run and shows a summary.
- **Timed practice**: 30s, 60s, 120s, 300s or 600s. Your score is the number
  correct, and the best score is saved for each method, level and duration.
- **Methods & help**: the rule for every technique, plus a worked example of
  each one with fresh random numbers. The examples use the exact steps the
  trainer shows when you miss a problem. Press `p` to practise the method
  you're reading.
- **Soroban visualizer**: type a number and see it as beads.
- **Stats**: accuracy and average solve time per method, and timed bests.

When you answer wrong or skip (Tab), the full worked solution for that
technique appears. Press `w` to also see it after correct answers.

## Methods

| Method | Techniques |
|---|---|
| Trachtenberg | ×11, ×12, ×5, ×6, ×7, ×9, ×8, ×4, ×3 (digit + neighbour rules) |
| Cross multiplication | 2×2, 3×2, 3×3, 4×4 digits, column by column |
| Anzan (flash) | numbers flashed one at a time; single-digit sums with 5/10-complement bead moves, multi-digit sums, and mixed + and − |
| Vedic / base | squares ending in 5, squares near 50 and 100, products near 100, difference of squares, ×9/99/999, ×25, any 2-digit square |
| Quant staples | fraction → percent, percent of a number, estimating products and quotients, squares up to 150, powers of 2, expected value of a bet |

Difficulty (1–5) controls the number of digits, how far numbers are from the
base, and how many numbers are flashed. In estimation drills it also narrows
the tolerance.

## Keys in a session

| Key | Action |
|---|---|
| `0-9 . - /` | type the answer (`3/8`, `37.5`, `-12` all work) |
| Enter | submit |
| Tab | skip and show the worked solution |
| `?` | show the rule for the current technique |
| `w` | show walkthroughs after correct answers too |
| `[` / `]` | flash slower / faster (anzan) |
| `r` | replay the flash (anzan) |
| `s` | show flashed numbers on a soroban (anzan) |
| Esc | end the session |

## Data

Stats are saved as JSON in the platform data directory (for example
`~/.local/share/mentalmath/stats.json` on Linux). To use a different
directory, set `MENTALMATH_DATA_DIR`. If the file is unreadable, it is moved
to `stats.json.corrupt` so it doesn't get overwritten.

## Development

```
cargo test     # every technique is checked against brute-force arithmetic
cargo clippy --all-targets -- -D warnings
```

The generators live in `src/methods/`, one module per method. Each technique
builds its answer by running the actual algorithm (for example, the
Trachtenberg neighbour rule digit by digit), and the tests compare that answer
with plain arithmetic.
