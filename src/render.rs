//! Maps sim state to the screen. Punch-Out layout: the opponent large at the
//! top facing you, Minimal small at the bottom seen from behind.
//! No game logic lives here.

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Gauge, Paragraph};
use ratatui::Frame;

use crate::sim::{Action, Fighter, HitKind, Match, Phase, Side, State, MAX_STAMINA, TICK_HZ};

pub struct View {
    pub show_debug: bool,
    pub show_hints: bool,
    pub last_events: Vec<String>,
}

pub fn draw(f: &mut Frame, m: &Match, view: &View) {
    let area = f.area();
    let rows = Layout::vertical([
        Constraint::Length(3), // bars
        Constraint::Length(1), // clock
        Constraint::Min(17),   // ring
        Constraint::Length(1), // help / debug
    ])
    .split(area);

    draw_bars(f, rows[0], m);
    draw_clock(f, rows[1], m);
    draw_ring(f, rows[2], m, view);
    draw_footer(f, rows[3], m, view);
}

// ---------------------------------------------------------------- top strip

fn health_color(ratio: f64) -> Color {
    if ratio > 0.5 {
        Color::Green
    } else if ratio > 0.25 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn fighter_bars(f: &mut Frame, area: Rect, fi: &Fighter, right: bool) {
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    let mut spans = vec![Span::styled(fi.name, Style::new().bold())];
    if fi.exhausted() {
        spans.push(Span::raw("  EXHAUSTED").red().bold());
    }
    if fi.counter_window > 0 {
        spans.push(Span::raw("  COUNTER!").magenta().bold());
    }
    let title = Line::from(spans);
    let title = if right { title.alignment(Alignment::Right) } else { title };
    f.render_widget(Paragraph::new(title), rows[0]);

    let hr = (fi.health / fi.max_health).clamp(0.0, 1.0) as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::new().fg(health_color(hr)).bg(Color::DarkGray))
            .ratio(hr)
            .label(format!("HP {:>3.0}", fi.health)),
        rows[1],
    );
    let sr = (fi.stamina / MAX_STAMINA).clamp(0.0, 1.0) as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::new().fg(Color::Blue).bg(Color::DarkGray))
            .ratio(sr)
            .label(format!("ST {:>3.0}", fi.stamina)),
        rows[2],
    );
}

fn draw_bars(f: &mut Frame, area: Rect, m: &Match) {
    let halves = Layout::horizontal([
        Constraint::Percentage(44),
        Constraint::Percentage(12),
        Constraint::Percentage(44),
    ])
    .split(area);
    fighter_bars(f, halves[0], &m.player, false);
    fighter_bars(f, halves[2], &m.opp, true);
    let secs = m.round_ticks_left / TICK_HZ;
    let clock = Text::from(vec![
        Line::from(format!("ROUND {}", m.round)).bold(),
        Line::from(format!("{}:{:02}", secs / 60, secs % 60)).bold(),
    ]);
    f.render_widget(Paragraph::new(clock).alignment(Alignment::Center), halves[1]);
}

fn draw_clock(f: &mut Frame, area: Rect, m: &Match) {
    let text = format!("MINIMAL  vs  {} \"{}\"", m.spec.name.to_uppercase(), m.spec.nickname);
    f.render_widget(Paragraph::new(text).alignment(Alignment::Center).dim(), area);
}

// ---------------------------------------------------------------- sprites

/// Where in the wind-up a fighter is: 0.0 just started, 1.0 about to land.
fn tell_progress(fi: &Fighter) -> Option<(Action, f32)> {
    if let State::Startup(a, left) = fi.state {
        let total = fi.startup_ticks(a).max(1) as f32;
        Some((a, 1.0 - (left as f32 / total).clamp(0.0, 1.0)))
    } else {
        None
    }
}

/// Chainguard, facing you. 27 columns, 11 rows. Picked by state.
fn opp_sprite(fi: &Fighter) -> &'static [&'static str] {
    use Action::*;
    if fi.hit_flash > 0 && !matches!(fi.state, State::Stunned(_)) {
        return &OPP_HIT;
    }
    match fi.state {
        State::Idle => &OPP_IDLE,
        State::Startup(Jab, _) => &OPP_JAB_WINDUP,
        State::Recovery(Jab, _) => &OPP_JAB,
        State::Startup(Hook, _) => &OPP_HOOK_WINDUP,
        State::Recovery(Hook, _) => &OPP_HOOK,
        State::Startup(Uppercut, _) => &OPP_UPPER_WINDUP,
        State::Recovery(Uppercut, _) => &OPP_UPPER,
        State::Startup(Special, _) => &OPP_SPECIAL_WINDUP,
        State::Recovery(Special, _) => &OPP_SPECIAL,
        State::Blocking(_) => &OPP_BLOCK,
        State::Dodging(..) => &OPP_IDLE,
        State::Stunned(_) => &OPP_STUNNED,
    }
}

const OPP_IDLE: [&str; 11] = [
    "         .---------.         ",
    "        /  _     _  \\        ",
    "       |  (o)   (o)  |       ",
    "       |      >      |       ",
    "        \\   .___.   /        ",
    "    .--. '---------' .--.    ",
    "   ( () )|           |( () )   ",
    "    '--' |   0 CVE   | '--'    ",
    "      \\__|           |__/      ",
    "         |___________|       ",
    "         |   |   |   |       ",
];

const OPP_JAB_WINDUP: [&str; 11] = [
    "         .---------.    .--. ",
    "        /  _     _  \\  ( () )",
    "       |  (o)   (o)  |  '--' ",
    "       |      >      |   ||  ",
    "        \\   .___.   /    ||  ",
    "    .--. '---------'____//   ",
    "   ( () )|           |       ",
    "    '--' |   0 CVE   |       ",
    "      \\__|           |       ",
    "         |___________|       ",
    "         |   |   |   |       ",
];

const OPP_JAB: [&str; 11] = [
    "         .---------.         ",
    "        /  _     _  \\        ",
    "       |  (o)   (o)  |       ",
    "       |      >   .--------. ",
    "        \\   .___ (    ()    )",
    "    .--. '------- '--------' ",
    "   ( () )|           |       ",
    "    '--' |   0 CVE   |       ",
    "      \\__|           |       ",
    "         |___________|       ",
    "         |   |   |   |       ",
];

const OPP_HOOK_WINDUP: [&str; 11] = [
    "             .---------.     ",
    "            /  _     _  \\    ",
    "           |  (o)   (o)  |   ",
    "           |      <      |   ",
    "            \\   .___.   /    ",
    " .--.       '---------'  .--.",
    "( () )     |           |( () )",
    " '--'______|   0 CVE   | '--'",
    "           |           |__/  ",
    "           |___________|     ",
    "           |   |   |   |     ",
];

const OPP_HOOK: [&str; 11] = [
    "         .---------.         ",
    "        /  _     _  \\        ",
    "       |  (o)   (o)  |       ",
    "  .--------.  >      |       ",
    " (    ()    )__.   /         ",
    "  '--------'-------' .--.    ",
    "         |           |( () )  ",
    "         |   0 CVE   | '--'   ",
    "         |           |__/     ",
    "         |___________|        ",
    "         |   |   |   |        ",
];

const OPP_UPPER_WINDUP: [&str; 11] = [
    "                             ",
    "                             ",
    "         .---------.         ",
    "        /  _     _  \\        ",
    "       |  (o)   (o)  |       ",
    "       |      v      |       ",
    "        \\   .___.   /        ",
    "         '---------'         ",
    "         |   0 CVE   |       ",
    "   .--.  |           |  .--. ",
    "  ( () )_|___________|_( () )",
];

const OPP_UPPER: [&str; 11] = [
    "            .--.             ",
    "           ( () )            ",
    "            '--'             ",
    "         .---||----.         ",
    "        /  _ ||  _  \\        ",
    "       |  (o)||(o)   |       ",
    "       |     ||      |       ",
    "        \\   .||_.   /        ",
    "         '---||----'         ",
    "         |   0 CVE   |  .--. ",
    "         |___________|_( () )",
];

const OPP_BLOCK: [&str; 11] = [
    "         .---------.         ",
    "        /.--.   .--.\\        ",
    "       |( () ) ( () )|       ",
    "       | '--'   '--' |       ",
    "        \\   .___.   /        ",
    "         '---------'         ",
    "         |           |       ",
    "         |   0 CVE   |       ",
    "         |           |       ",
    "         |___________|       ",
    "         |   |   |   |       ",
];

const OPP_STUNNED: [&str; 11] = [
    "      *  .---------.  *      ",
    "        /  _     _  \\        ",
    "       |  (x)   (x)  |   *   ",
    "       |      o      |       ",
    "   *    \\   .___.   /        ",
    "         '---------'         ",
    "    .--. |           | .--.  ",
    "   ( () )|   0 CVE   |( () ) ",
    "    '--' |           | '--'  ",
    "         |___________|       ",
    "         |   |   |   |       ",
];

const OPP_SPECIAL_WINDUP: [&str; 11] = [
    "         .---------.    _____",
    "        /  _     _  \\  |$$$$$|",
    "       |  ($)   ($)  | |QUOTE|",
    "       |      >      | |_____|",
    "        \\   .___.   /    ||  ",
    "    .--. '---------'____//   ",
    "   ( () )|           |       ",
    "    '--' |   0 CVE   |       ",
    "      \\__|           |       ",
    "         |___________|       ",
    "         |   |   |   |       ",
];

const OPP_SPECIAL: [&str; 11] = [
    "         .---------.         ",
    "        /  _     _  \\        ",
    "       |  ($)   ($)  |       ",
    "       .-------------.       ",
    "      (               )      ",
    "     (     ( $$$ )     )     ",
    "      (               )      ",
    "       '-------------'       ",
    "         |   0 CVE   |       ",
    "         |___________|       ",
    "         |   |   |   |       ",
];

const OPP_HIT: [&str; 11] = [
    "            .---------.      ",
    "           /  _     _  \\     ",
    "          |  (>)   (<)  |    ",
    "          |      o      |    ",
    "           \\   .___.   /     ",
    "    .--.    '---------'.--.  ",
    "   ( () )  |           ( () )",
    "    '--'   |   0 CVE   |'--' ",
    "      \\____|           |     ",
    "           |___________|     ",
    "           |   |   |   |     ",
];

/// Minimal, seen from behind. 13 columns, 5 rows.
fn player_sprite(fi: &Fighter) -> &'static [&'static str] {
    use Action::*;
    match fi.state {
        State::Idle => &P_IDLE,
        State::Startup(Jab, _) | State::Recovery(Jab, _) => &P_JAB,
        State::Startup(Hook, _) | State::Recovery(Hook, _) => &P_HOOK,
        State::Startup(Uppercut, _) | State::Recovery(Uppercut, _) => &P_UPPER,
        State::Startup(Special, _) | State::Recovery(Special, _) => &P_UPPER,
        State::Blocking(_) => &P_BLOCK,
        State::Dodging(Side::Left, _) => &P_DODGE_L,
        State::Dodging(Side::Right, _) => &P_DODGE_R,
        State::Stunned(_) => &P_STUNNED,
    }
}

const P_IDLE: [&str; 5] = [
    "    .---.    ",
    "   /     \\   ",
    "  (O)   (O)  ",
    "   |     |   ",
    "   |_____|   ",
];
const P_JAB: [&str; 5] = [
    "          (O)",
    "    .---. /  ",
    "   /     \\   ",
    "  (O)    |   ",
    "   |_____|   ",
];
const P_HOOK: [&str; 5] = [
    " (O)         ",
    "   \\.---.    ",
    "   /     \\   ",
    "   |    (O)  ",
    "   |_____|   ",
];
const P_UPPER: [&str; 5] = [
    "     (O)     ",
    "    .-|-.    ",
    "   /     \\   ",
    "  (O)    |   ",
    "   |_____|   ",
];
const P_BLOCK: [&str; 5] = [
    "   (O) (O)   ",
    "    .---.    ",
    "   /     \\   ",
    "   |     |   ",
    "   |_____|   ",
];
const P_DODGE_L: [&str; 5] = [
    "  .---.      ",
    " /     \\     ",
    "(O)   (O)    ",
    " \\     \\     ",
    "  \\_____\\    ",
];
const P_DODGE_R: [&str; 5] = [
    "      .---.  ",
    "     /     \\ ",
    "    (O)   (O)",
    "     /     / ",
    "    /_____/  ",
];
const P_STUNNED: [&str; 5] = [
    "  * .---. *  ",
    "   /     \\   ",
    "   | ~ ~ |   ",
    "  (O)   (O)  ",
    "   |_____|   ",
];

// ---------------------------------------------------------------- ring

/// Write a sprite at (x, y) with one style, clipped to `area`.
fn blit(buf: &mut Buffer, area: Rect, x: i32, y: i32, lines: &[&str], style: Style) {
    for (i, line) in lines.iter().enumerate() {
        let yy = y + i as i32;
        if yy < area.y as i32 || yy >= area.bottom() as i32 {
            continue;
        }
        for (j, ch) in line.chars().enumerate() {
            if ch == ' ' {
                continue; // transparent
            }
            let xx = x + j as i32;
            if xx < area.x as i32 || xx >= area.right() as i32 {
                continue;
            }
            buf[(xx as u16, yy as u16)].set_char(ch).set_style(style);
        }
    }
}

fn blit_str(buf: &mut Buffer, area: Rect, x: i32, y: u16, s: &str, style: Style) {
    if y < area.y || y >= area.bottom() {
        return;
    }
    for (j, ch) in s.chars().enumerate() {
        let xx = x + j as i32;
        if xx < area.x as i32 || xx >= area.right() as i32 {
            continue;
        }
        buf[(xx as u16, y)].set_char(ch).set_style(style);
    }
}

fn opp_style(m: &Match) -> Style {
    let base = Style::new().fg(Color::White);
    if m.opp.hit_flash > 0 {
        return Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);
    }
    match tell_progress(&m.opp) {
        Some((_, p)) if p < 0.5 => Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        Some(_) => {
            // Flash in the second half of the wind-up: the punch is coming now.
            if (m.tick / 2) % 2 == 0 {
                Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::LightRed).add_modifier(Modifier::BOLD | Modifier::REVERSED)
            }
        }
        None => match m.opp.state {
            State::Stunned(_) => Style::new().fg(Color::Magenta),
            State::Blocking(_) => Style::new().fg(Color::Cyan),
            _ => base,
        },
    }
}

fn player_style(m: &Match) -> Style {
    if m.player.hit_flash > 0 {
        Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if m.player.counter_window > 0 {
        Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else if matches!(m.player.state, State::Stunned(_)) {
        Style::new().fg(Color::DarkGray)
    } else {
        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
    }
}

fn punch_name(a: Action) -> &'static str {
    match a {
        Action::Jab => "JAB",
        Action::Hook => "HOOK",
        Action::Uppercut => "UPPERCUT",
        Action::Special => "SPECIAL",
    }
}

fn draw_ring(f: &mut Frame, area: Rect, m: &Match, view: &View) {
    let cx = area.x as i32 + area.width as i32 / 2;
    let buf = f.buffer_mut();

    // Canvas ropes.
    let rope = Style::new().fg(Color::DarkGray);
    let rope_line: String = "═".repeat(area.width as usize);
    blit_str(buf, area, area.x as i32, area.y + 10, &rope_line, rope);
    blit_str(buf, area, area.x as i32, area.y + 12, &rope_line, rope);

    // Opponent: 27 wide, 11 tall, top of the ring.
    let opp = opp_sprite(&m.opp);
    blit(buf, area, cx - 14, area.y as i32, opp, opp_style(m));

    // Tell fuse and hint under the opponent's feet.
    if let Some((a, p)) = tell_progress(&m.opp) {
        let width = 21usize;
        let filled = (p * width as f32).round() as usize;
        let fuse = format!("[{}{}]", "█".repeat(filled), "░".repeat(width - filled));
        let color = if p < 0.5 { Color::Yellow } else { Color::Red };
        blit_str(buf, area, cx - 11, area.y + 11, &fuse, Style::new().fg(color).bold());
        if view.show_hints {
            let hint = match a {
                Action::Jab => "jab coming: block or dodge",
                Action::Hook => "HOOK coming: dodge",
                Action::Uppercut => "UPPERCUT coming: dodge, do not block",
                Action::Special => "ENTERPRISE QUOTE: DODGE IT",
            };
            let x = cx - hint.chars().count() as i32 / 2;
            blit_str(buf, area, x, area.y + 13, hint, Style::new().fg(color).bold());
        }
        let name = punch_name(a);
        blit_str(buf, area, cx - 14 - name.len() as i32 - 2, area.y + 5, name, Style::new().fg(color).bold());
        blit_str(buf, area, cx + 15, area.y + 5, name, Style::new().fg(color).bold());
    }

    // Enterprise Quote price counter beside the briefcase.
    if let State::Startup(Action::Special, left) = m.opp.state {
        let total = m.opp.startup_ticks(Action::Special).max(1) as f32;
        let progress = 1.0 - (left as f32 / total).clamp(0.0, 1.0);
        let price = (progress * 250_000.0) as u32;
        let s = format!("${}/yr", fmt_thousands(price));
        blit_str(buf, area, cx + 16, area.y + 1, &s, Style::new().fg(Color::Yellow).bold());
    }

    // Player: 13 wide, 5 tall, bottom of the ring, seen from behind.
    let p = player_sprite(&m.player);
    let dodge_dx = match m.player.state {
        State::Dodging(Side::Left, _) => -5,
        State::Dodging(Side::Right, _) => 5,
        _ => 0,
    };
    blit(buf, area, cx - 6 + dodge_dx, area.y as i32 + 14, p, player_style(m));

    // Hit text: damage numbers float beside whoever got hit.
    for ev in &m.events {
        let (txt, color) = match ev.kind {
            HitKind::Hit => (format!("-{:.0}", ev.damage), Color::White),
            HitKind::Counter => (format!("COUNTER -{:.0}!", ev.damage), Color::Magenta),
            HitKind::Chip => (format!("blocked -{:.0}", ev.damage), Color::Cyan),
            HitKind::GuardBreak => (format!("GUARD BREAK -{:.0}", ev.damage), Color::Red),
            HitKind::Interrupt => (format!("interrupt -{:.0}", ev.damage), Color::Yellow),
            HitKind::Dodged => ("whiff".to_string(), Color::Green),
            HitKind::Stagger => (format!("STAGGER -{:.0}", ev.damage), Color::Red),
        };
        let style = Style::new().fg(color).add_modifier(Modifier::BOLD);
        if ev.by_player {
            // Landed on the opponent: show by the opponent's head.
            blit_str(buf, area, cx + 16, area.y + 3, &txt, style);
        } else {
            blit_str(buf, area, cx + 9, area.y + 15, &txt, style);
        }
    }

    // Side column: recent events for the debug view.
    if view.show_debug {
        for (i, s) in view.last_events.iter().rev().take(6).enumerate() {
            blit_str(buf, area, area.x as i32 + 1, area.y + 1 + i as u16, s, Style::new().dim());
        }
    }

    // Banner: round intro, taunts, KO. Centred over the ropes.
    if let Some((s, _)) = &m.banner {
        let (head, body, tail) = match m.phase {
            Phase::Over { player_won } => (
                Some(if player_won { "YOU WIN" } else { "YOU LOSE" }),
                s.as_str(),
                Some("[r] rematch   [q] quit"),
            ),
            _ => (None, s.as_str(), None),
        };
        let y0 = area.y + 10;
        let style = Style::new().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD);
        let w = body.chars().count().max(head.map_or(0, |h| h.len())).max(tail.map_or(0, |t| t.len())) + 4;
        let x = cx - w as i32 / 2;
        let pad = |t: &str| format!("{:^w$}", t, w = w);
        let mut y = y0;
        if let Some(h) = head {
            blit_str(buf, area, x, y, &pad(h), style);
            y += 1;
        }
        blit_str(buf, area, x, y, &pad(body), style);
        if let Some(t) = tail {
            blit_str(buf, area, x, y + 1, &pad(t), style.remove_modifier(Modifier::BOLD));
        }
    }
}

fn fmt_thousands(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn draw_footer(f: &mut Frame, area: Rect, m: &Match, view: &View) {
    let text = if view.show_debug {
        format!("t={} p={:?} o={:?}", m.tick, m.player.state, m.opp.state)
    } else {
        "[j] jab  [k] hook  [l] uppercut  [space] block  [a]/[d] dodge  [h] hints  [tab] debug  [q] quit".to_string()
    };
    f.render_widget(Paragraph::new(text).dim().alignment(Alignment::Center), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{chainguard, Input, INTRO_TICKS};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render(m: &Match) -> String {
        let view = View { show_debug: false, show_hints: true, last_events: vec![] };
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| draw(f, m, &view)).unwrap();
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

    fn fighting() -> Match {
        let mut m = Match::new(chainguard(), 3);
        for _ in 0..(INTRO_TICKS + 40) {
            m.step();
        }
        m.banner = None;
        m
    }

    #[test]
    fn renders_hook_tell_at_80x24() {
        let mut m = fighting();
        m.opp.state = State::Startup(Action::Hook, 6);
        m.player_input(Input::Block);
        let out = render(&m);
        println!("{out}");
        assert!(out.contains("Minimal"));
        assert!(out.contains("HOOK coming"));
    }

    #[test]
    fn renders_special_and_player_jab() {
        let mut m = fighting();
        m.opp.state = State::Startup(Action::Special, 30);
        m.player_input(Input::Jab);
        let out = render(&m);
        println!("{out}");
        assert!(out.contains("QUOTE"));
        assert!(out.contains("$"));
    }

    #[test]
    fn renders_ko_banner() {
        let mut m = fighting();
        m.opp.health = 1.0;
        m.player.state = State::Startup(Action::Hook, 1);
        m.step();
        let out = render(&m);
        println!("{out}");
        assert!(out.contains("YOU WIN"));
    }
}
