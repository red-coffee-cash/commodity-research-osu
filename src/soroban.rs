//! Soroban (Japanese abacus) model. Each rod holds one digit: the heaven bead
//! above the beam is worth 5 when pushed down to it, and each of the four earth
//! beads below is worth 1 when pushed up to it.

pub const BEAD: char = '●';
pub const ROD: char = '│';
pub const BEAM: char = '═';

/// Bead state of one rod.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rod {
    pub heaven: bool,
    pub earth: u8,
}

impl Rod {
    pub fn from_digit(d: u8) -> Self {
        assert!(d <= 9);
        Rod {
            heaven: d >= 5,
            earth: d % 5,
        }
    }

    pub fn value(self) -> u8 {
        if self.heaven { 5 + self.earth } else { self.earth }
    }

    /// The rod drawn top to bottom: 2 heaven slots, the beam, 5 earth slots.
    pub fn column(self) -> [char; 8] {
        let mut c = [ROD; 8];
        // The inactive heaven bead sits at the top; the active one touches the beam.
        c[if self.heaven { 1 } else { 0 }] = BEAD;
        c[2] = BEAM;
        // Active earth beads touch the beam, then a gap, then the rest.
        for slot in 0..5u8 {
            if slot != self.earth {
                c[3 + slot as usize] = BEAD;
            }
        }
        c
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_column(c: &[char; 8]) -> Self {
        let heaven = c[1] == BEAD;
        let earth = (0..5).find(|&s| c[3 + s] != BEAD).unwrap_or(4) as u8;
        Rod { heaven, earth }
    }
}

/// Rods for `n`, most significant first, padded on the left to `min_rods`.
pub fn rods(n: u64, min_rods: usize) -> Vec<Rod> {
    let s = n.to_string();
    let mut out: Vec<Rod> = vec![Rod::from_digit(0); min_rods.saturating_sub(s.len())];
    out.extend(s.bytes().map(|b| Rod::from_digit(b - b'0')));
    out
}

/// Text rows of the soroban for `n`, framed, with the digit under each rod.
pub fn render(n: u64, min_rods: usize) -> Vec<String> {
    let rs = rods(n, min_rods);
    let cols: Vec<[char; 8]> = rs.iter().map(|r| r.column()).collect();
    let width = rs.len() * 3;
    let mut rows = vec![format!("┌{}┐", "─".repeat(width))];
    for i in 0..8 {
        let fill = if i == 2 { BEAM } else { ' ' };
        let mut line = String::from(if i == 2 { "╞" } else { "│" });
        for c in &cols {
            line.push(fill);
            line.push(c[i]);
            line.push(fill);
        }
        line.push(if i == 2 { '╡' } else { '│' });
        rows.push(line);
    }
    rows.push(format!("└{}┘", "─".repeat(width)));
    let digits: String = rs.iter().map(|r| format!(" {} ", r.value())).collect();
    rows.push(format!(" {digits} "));
    rows
}

/// Read the value back off rendered rows (used to check the renderer).
#[cfg_attr(not(test), allow(dead_code))]
pub fn decode(rows: &[String]) -> u64 {
    let grid: Vec<Vec<char>> = rows[1..9].iter().map(|r| r.chars().collect()).collect();
    let rods = (grid[0].len() - 2) / 3;
    let mut n = 0u64;
    for r in 0..rods {
        let x = 2 + r * 3;
        let mut col = [' '; 8];
        for (i, row) in grid.iter().enumerate() {
            col[i] = row[x];
        }
        n = n * 10 + Rod::from_column(&col).value() as u64;
    }
    n
}

/// "7 = 5 + 2" style breakdown for each digit.
pub fn explain(n: u64) -> Vec<String> {
    n.to_string()
        .bytes()
        .map(|b| {
            let d = b - b'0';
            match (d >= 5, d % 5) {
                (false, 0) => "0: all beads away from the beam".to_string(),
                (false, e) => format!("{d}: {e} earth bead(s) up"),
                (true, 0) => format!("{d}: heaven bead down"),
                (true, e) => format!("{d}: heaven bead down (5) + {e} earth up"),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for n in (0..100_000u64).step_by(7).chain([0, 5, 9, 99_999, 1_234_567_890]) {
            let rows = render(n, 6);
            assert_eq!(decode(&rows), n, "{}", rows.join("\n"));
        }
    }

    #[test]
    fn column_shape() {
        // 7: heaven down, 2 earth up, gap, 2 earth down.
        assert_eq!(
            Rod::from_digit(7).column(),
            ['│', '●', '═', '●', '●', '│', '●', '●']
        );
        assert_eq!(
            Rod::from_digit(0).column(),
            ['●', '│', '═', '│', '●', '●', '●', '●']
        );
    }
}
