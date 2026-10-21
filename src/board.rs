//! The card between fights: eight portraits in ladder order, the way the
//! SNES fighters showed the world map. Beaten opponents are stamped KO, the
//! next one flashes, everyone further up is a silhouette.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::art;
use crate::pixel::{Canvas, Rgb};
use crate::sim::{Fighter, OpponentSpec};

/// Logical board size at scale 1. Four tiles across, two rows down.
pub const BOARD_W: i32 = 100;
pub const BOARD_H: i32 = 72;
const COLS: usize = 4;
const TILE_W: i32 = 24;
const TILE_H: i32 = 28;
const GAP: i32 = 1;
/// Pixels under each tile row for the two text labels (two cells at scale 1).
const LABEL_H: i32 = 6;

const BG: Rgb = Rgb::hex(0x16112b);
const LOCKED_BG: Rgb = Rgb::hex(0x1f1a33);
const FRAME: Rgb = Rgb::hex(0x3a3550);
const WHITE: Rgb = Rgb::hex(0xffffff);
const YELLOW: Rgb = Rgb::hex(0xffd23f);
const RED: Rgb = Rgb::hex(0xff3b3b);
const SILHOUETTE: Rgb = Rgb::hex(0x0c0a18);

/// Progress through the ladder. Index is ladder position, easiest first.
pub struct Board {
    pub defeated: Vec<bool>,
    /// Slots that can be picked: everyone beaten, plus the next one up.
    pub unlocked: usize,
    pub cursor: usize,
    pub tick: u64,
}

impl Board {
    pub fn new(n: usize) -> Board {
        Board { defeated: vec![false; n], unlocked: 1, cursor: 0, tick: 0 }
    }

    pub fn champion(&self) -> bool {
        self.defeated.iter().all(|d| *d)
    }

    /// First opponent not yet beaten.
    pub fn next_up(&self) -> Option<usize> {
        self.defeated.iter().position(|d| !d)
    }

    /// Record a win and move the cursor on to the next one.
    pub fn beat(&mut self, i: usize) {
        self.defeated[i] = true;
        if let Some(n) = self.next_up() {
            self.cursor = n;
            self.unlocked = self.unlocked.max(n + 1);
        }
    }

    /// Dev shortcut: make slot `i` pickable and point at it.
    pub fn unlock_to(&mut self, i: usize) {
        self.unlocked = self.unlocked.max(i + 1).min(self.defeated.len());
        self.cursor = i;
    }

    pub fn move_cursor(&mut self, d: i32) {
        let n = self.unlocked.max(1) as i32;
        self.cursor = (self.cursor as i32 + d).rem_euclid(n) as usize;
    }
}

pub fn draw(f: &mut Frame, b: &Board, specs: &[OpponentSpec]) {
    let area = f.area();
    let rows = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Min(36),   // tiles
        Constraint::Length(1), // status
        Constraint::Length(1), // hints
    ])
    .split(area);

    // Every win proves one thing in Minimal's voice; the title keeps score.
    let proved: Vec<&str> = specs.iter().zip(&b.defeated).filter(|(_, d)| **d).map(|(s, _)| s.special_line).collect();
    let mut spans = vec![
        Span::styled("MINIMAL PUNCH-OUT", Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("THE CARD", Style::new().fg(Color::White)),
    ];
    if !proved.is_empty() {
        spans.push(Span::styled("   PROVED: ", Style::new().fg(Color::Gray)));
        spans.push(Span::styled(proved.join(" / "), Style::new().fg(Color::Rgb(61, 220, 151)).add_modifier(Modifier::BOLD)));
    }
    let title = Line::from(spans);
    f.render_widget(Paragraph::new(title).alignment(Alignment::Center), rows[0]);

    let tiles = rows[1];
    let scale = ((tiles.width as i32 / BOARD_W).min(tiles.height as i32 * 2 / BOARD_H)).max(1);
    let w = (tiles.width as i32 / scale).max(BOARD_W);
    let h = (tiles.height as i32 * 2 / scale).max(BOARD_H);
    let (c, origin) = paint(b, specs, w, h);
    c.flush(f.buffer_mut(), tiles, scale);

    // Names under the tiles as crisp cell text.
    let (ox, oy) = origin;
    for (i, spec) in specs.iter().enumerate() {
        let (tx, ty) = tile_at(i, ox, oy);
        let cx = tiles.x as i32 + tx * scale;
        let cy = tiles.y as i32 + (ty + TILE_H) * scale / 2;
        let wcells = (TILE_W * scale) as u16;
        if cy + 1 >= tiles.bottom() as i32 || cx < 0 {
            continue;
        }
        let locked = i >= b.unlocked;
        let name_style = if i == b.cursor {
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if locked {
            Style::new().fg(Color::DarkGray)
        } else if b.defeated[i] {
            Style::new().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD)
        };
        let nick_style = if locked { Style::new().fg(Color::DarkGray) } else { Style::new().fg(Color::Gray) };
        let name = Paragraph::new(spec.name.to_uppercase()).style(name_style).alignment(Alignment::Center);
        let nick = Paragraph::new(spec.nickname).style(nick_style).alignment(Alignment::Center);
        f.render_widget(name, Rect { x: cx as u16, y: cy as u16, width: wcells, height: 1 });
        f.render_widget(nick, Rect { x: cx as u16, y: cy as u16 + 1, width: wcells, height: 1 });
    }

    let status = if b.champion() {
        Line::from("THE CARD IS CLEARED. Nobody left to fight but rematches.").bold().yellow()
    } else {
        let s = &specs[b.cursor];
        let verb = if b.defeated[b.cursor] { "REMATCH" } else { "NEXT UP" };
        Line::from(vec![
            Span::styled(format!("{verb}: "), Style::new().fg(Color::Gray)),
            Span::styled(s.name.to_uppercase(), Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  \"{}\"   rank {} of {}", s.nickname, s.rank, specs.len()), Style::new().fg(Color::Gray)),
        ])
    };
    f.render_widget(Paragraph::new(status).alignment(Alignment::Center), rows[2]);
    let hints = if b.unlocked > 1 { "[enter] fight   [a] [d] pick   [q] quit" } else { "[enter] fight   [q] quit" };
    f.render_widget(Paragraph::new(hints).dim().alignment(Alignment::Center), rows[3]);
}

fn tile_at(i: usize, ox: i32, oy: i32) -> (i32, i32) {
    let col = (i % COLS) as i32;
    let row = (i / COLS) as i32;
    (ox + col * (TILE_W + GAP), oy + row * (TILE_H + LABEL_H + GAP))
}

/// Paint the tiles into a w x h canvas. Returns the canvas and the origin of
/// the tile grid, so the caller can place cell text under each tile.
pub fn paint(b: &Board, specs: &[OpponentSpec], w: i32, h: i32) -> (Canvas, (i32, i32)) {
    let mut c = Canvas::new(w, h, BG);
    let grid_w = COLS as i32 * TILE_W + (COLS as i32 - 1) * GAP;
    let rows = (specs.len() + COLS - 1) / COLS;
    let grid_h = rows as i32 * (TILE_H + LABEL_H + GAP) - GAP;
    let ox = (w - grid_w) / 2;
    let oy = ((h - grid_h) / 2).max(1);

    // Faint rope lines behind the tiles so it still reads as a ring.
    for y in (0..h).step_by(9) {
        c.rect(0, y, w, 1, BG.mix(WHITE, 0.05));
    }

    let model = Fighter::new("portrait");
    for (i, spec) in specs.iter().enumerate() {
        let (tx, ty) = tile_at(i, ox, oy);
        let look = art::look(spec.id);
        let locked = i >= b.unlocked;
        let bg = if locked { LOCKED_BG } else { look.banner.darker(0.35) };
        c.rect(tx, ty, TILE_W, TILE_H, bg);

        // Portrait: the top of the rig, cropped to head and shoulders.
        let mut rig = Canvas::new(44, 56, bg);
        if locked {
            rig.set_tint(Some((SILHOUETTE, 0.92)));
        } else if b.defeated[i] {
            rig.set_tint(Some((bg, 0.55)));
        }
        let pose = art::opp_pose(&model, if i == b.cursor { b.tick } else { 0 }, spec.id);
        art::draw_opponent(&mut rig, 0, 0, &look, &pose, b.tick);
        c.blit(&rig, 22 - TILE_W / 2, 0, TILE_W, TILE_H, tx, ty);

        if b.defeated[i] {
            // KO stamp: a red cross and the letters.
            c.line(tx + 2, ty + 2, tx + TILE_W - 3, ty + TILE_H - 3, 1.2, RED);
            c.line(tx + TILE_W - 3, ty + 2, tx + 2, ty + TILE_H - 3, 1.2, RED);
            let tw = Canvas::text_width("KO", 2);
            c.rect(tx + TILE_W / 2 - tw / 2 - 2, ty + TILE_H / 2 - 6, tw + 4, 12, Rgb::hex(0x14141c));
            c.text(tx + TILE_W / 2 - tw / 2, ty + TILE_H / 2 - 5, "KO", 2, RED);
        }
        if locked {
            let tw = Canvas::text_width("?", 2);
            c.text(tx + TILE_W / 2 - tw / 2, ty + TILE_H / 2 - 5, "?", 2, FRAME.mix(WHITE, 0.3));
        }

        // Frame, rank badge, and the flashing cursor.
        let frame = if i == b.cursor {
            if (b.tick / 6) % 2 == 0 { YELLOW } else { WHITE }
        } else {
            FRAME
        };
        c.rect(tx, ty, TILE_W, 1, frame);
        c.rect(tx, ty + TILE_H - 1, TILE_W, 1, frame);
        c.rect(tx, ty, 1, TILE_H, frame);
        c.rect(tx + TILE_W - 1, ty, 1, TILE_H, frame);
        if i == b.cursor {
            c.rect(tx - 1, ty - 1, TILE_W + 2, 1, frame);
            c.rect(tx - 1, ty + TILE_H, TILE_W + 2, 1, frame);
            c.rect(tx - 1, ty - 1, 1, TILE_H + 2, frame);
            c.rect(tx + TILE_W, ty - 1, 1, TILE_H + 2, frame);
        }
        let badge = format!("{}", spec.rank);
        c.rect(tx + 1, ty + 1, 5, 7, Rgb::hex(0x14141c));
        c.text(tx + 2, ty + 2, &badge, 1, if locked { FRAME.mix(WHITE, 0.4) } else { WHITE });
    }
    (c, (ox, oy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render(b: &Board, cols: u16, rows: u16) -> String {
        let specs = crate::roster::all();
        let mut t = Terminal::new(TestBackend::new(cols, rows)).unwrap();
        t.draw(|f| draw(f, b, &specs)).unwrap();
        let buf = t.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn fresh_card_points_at_the_first_opponent() {
        let b = Board::new(8);
        let s = render(&b, 100, 40);
        assert!(s.contains("NEXT UP: CODESPACES"), "{s}");
        assert!(s.contains("CODESPACES"));
        assert!(s.contains("NIX"));
    }

    #[test]
    fn a_win_moves_the_cursor_on() {
        let mut b = Board::new(8);
        b.beat(0);
        assert_eq!(b.cursor, 1);
        assert_eq!(b.unlocked, 2);
        b.move_cursor(-1);
        assert_eq!(b.cursor, 0);
        let s = render(&b, 100, 40);
        assert!(s.contains("REMATCH: CODESPACES"), "{s}");
        for i in 0..8 {
            b.beat(i);
        }
        assert!(b.champion());
        assert!(render(&b, 120, 50).contains("THE CARD IS CLEARED"));
    }

    #[test]
    fn dump_board_frames() {
        let Ok(dir) = std::env::var("PUNCHOUT_FRAMES") else { return };
        let specs = crate::roster::all();
        let mut b = Board::new(8);
        crate::pixel::png::write(&paint(&b, &specs, BOARD_W, BOARD_H).0, &format!("{dir}/board_start.png"), 6);
        b.beat(0);
        b.beat(1);
        b.beat(2);
        b.tick = 6;
        crate::pixel::png::write(&paint(&b, &specs, BOARD_W, BOARD_H).0, &format!("{dir}/board_mid.png"), 6);
    }
}
