//! Screen layout. The ring is a pixel canvas rendered through half-block
//! glyphs; bars, header and footer are ordinary ratatui widgets.
//! No game logic lives here.

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Gauge, Paragraph};
use ratatui::Frame;

use crate::art;
use crate::pixel::{Canvas, Rgb};
use crate::sim::{Action, Fighter, HitKind, Match, Phase, State, MAX_STAMINA, TICK_HZ};

/// Smallest terminal we lay out for. Below this we ask for a bigger window.
pub const MIN_COLS: u16 = 100;
pub const MIN_ROWS: u16 = 40;

/// Logical ring size in pixels at scale 1.
const RING_W: i32 = 100;
const RING_H: i32 = 70;

pub struct View {
    pub show_debug: bool,
    pub show_hints: bool,
    pub last_events: Vec<String>,
}

pub fn draw(f: &mut Frame, m: &Match, view: &View) {
    let area = f.area();
    if area.width < MIN_COLS || area.height < MIN_ROWS {
        let msg = Text::from(vec![
            Line::from("MINIMAL PUNCH-OUT").bold(),
            Line::from(""),
            Line::from(format!("Terminal is {}x{}. Please enlarge it to at least {}x{}.", area.width, area.height, MIN_COLS, MIN_ROWS)),
            Line::from("[q] quit").dim(),
        ]);
        let y = area.y + area.height / 2 - 2;
        let r = Rect { x: area.x, y, width: area.width, height: 4.min(area.height) };
        f.render_widget(Paragraph::new(msg).alignment(Alignment::Center), r);
        return;
    }
    let rows = Layout::vertical([
        Constraint::Length(3), // bars
        Constraint::Length(1), // header
        Constraint::Min(35),   // ring
        Constraint::Length(1), // footer
    ])
    .split(area);

    draw_bars(f, rows[0], m);
    draw_header(f, rows[1], m);
    draw_ring(f, rows[2], m, view);
    draw_footer(f, rows[3], m, view);
}

// ---------------------------------------------------------------- top strip

fn health_color(ratio: f64) -> Color {
    if ratio > 0.5 {
        Color::Rgb(70, 200, 90)
    } else if ratio > 0.25 {
        Color::Rgb(240, 190, 60)
    } else {
        Color::Rgb(220, 50, 60)
    }
}

fn fighter_bars(f: &mut Frame, area: Rect, fi: &Fighter, right: bool) {
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    let mut spans = vec![Span::styled(fi.name.to_uppercase(), Style::new().bold())];
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
            .gauge_style(Style::new().fg(health_color(hr)).bg(Color::Rgb(50, 50, 60)))
            .ratio(hr)
            .label(format!("HP {:>3.0}", fi.health)),
        rows[1],
    );
    let sr = (fi.stamina / MAX_STAMINA).clamp(0.0, 1.0) as f64;
    // Blue while you can afford a dodge, orange when one more punch would
    // leave you unable to, red once you are there.
    let st_color = if fi.exhausted() {
        Color::Rgb(255, 59, 59)
    } else if fi.stamina < 30.0 {
        Color::Rgb(255, 150, 40)
    } else {
        Color::Rgb(70, 130, 240)
    };
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::new().fg(st_color).bg(Color::Rgb(50, 50, 60)))
            .ratio(sr)
            .label(format!("ST {:>3.0}", fi.stamina)),
        rows[2],
    );
}

fn draw_bars(f: &mut Frame, area: Rect, m: &Match) {
    let halves = Layout::horizontal([
        Constraint::Percentage(42),
        Constraint::Percentage(16),
        Constraint::Percentage(42),
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

fn draw_header(f: &mut Frame, area: Rect, m: &Match) {
    let text = format!("MINIMAL  vs  {} \"{}\"", m.spec.name.to_uppercase(), m.spec.nickname);
    f.render_widget(Paragraph::new(text).alignment(Alignment::Center).dim(), area);
}

// ---------------------------------------------------------------- ring

/// Where in the wind-up a fighter is: 0.0 just started, 1.0 about to land.
fn tell_progress(fi: &Fighter) -> Option<(Action, f32)> {
    if let State::Startup(a, left) = fi.state {
        let total = fi.startup_ticks(a).max(1) as f32;
        Some((a, 1.0 - (left as f32 / total).clamp(0.0, 1.0)))
    } else {
        None
    }
}

fn punch_name(a: Action) -> &'static str {
    match a {
        Action::Jab => "JAB",
        Action::Hook => "HOOK",
        Action::Uppercut => "UPPERCUT",
        Action::Special => "QUOTE",
    }
}

const CROWD: Rgb = Rgb::hex(0x16112b);
const FLOOR: Rgb = Rgb::hex(0xd9dde6);
const FLOOR_LINE: Rgb = Rgb::hex(0xb9bfcc);
const APRON: Rgb = Rgb::hex(0x2b3a6b);
const WHITE: Rgb = Rgb::hex(0xffffff);
const YELLOW: Rgb = Rgb::hex(0xffd23f);
const RED: Rgb = Rgb::hex(0xff3b3b);
const MAGENTA: Rgb = Rgb::hex(0xff5ad1);

fn draw_ring(f: &mut Frame, area: Rect, m: &Match, view: &View) {
    // Integer scale so the 100x70 logical ring fills as much as it can.
    let scale = ((area.width as i32 / RING_W).min(area.height as i32 * 2 / RING_H)).max(1);
    let w = (area.width as i32 / scale).max(RING_W);
    let h = (area.height as i32 * 2 / scale).max(RING_H);
    let c = paint_ring(m, w, h);
    c.flush(f.buffer_mut(), area, scale);
    overlay_text(f, area, m, view);
}

/// Paint the whole ring into a logical pixel canvas of w x h.
pub fn paint_ring(m: &Match, w: i32, h: i32) -> Canvas {
    let mut c = Canvas::new(w, h, CROWD);
    let cx = w / 2;

    // Crowd: a dark band with speckled heads.
    for y in 0..18 {
        for x in 0..w {
            let n = (x * 7919 + y * 104729) % 23;
            if n < 4 {
                let shade = CROWD.mix(Rgb::hex(0x8a7fb0), 0.25 + (n as f32) * 0.1);
                c.put(x, y, shade);
            }
        }
    }
    // Sponsor banner behind the opponent, so you always know who you fight.
    // During a Rebrand the name flickers through the aliases.
    let look = art::look(m.spec.id);
    let name = if m.rebrand > 0 && !m.spec.aliases.is_empty() {
        m.spec.aliases[(m.alias + (m.tick / 3) as usize) % m.spec.aliases.len()].to_uppercase()
    } else {
        m.opp_name().to_uppercase()
    };
    let tw = Canvas::text_width(&name, 2);
    c.rect(0, 2, w, 14, look.banner);
    c.rect(0, 2, w, 1, WHITE);
    c.rect(0, 15, w, 1, WHITE);
    let name_col = if m.rebrand > 0 && (m.tick / 3) % 2 == 0 { YELLOW } else { WHITE };
    c.text(cx - tw / 2, 5, &name, 2, name_col);
    for x in [cx - tw / 2 - 12, cx + tw / 2 + 12] {
        art::draw_logo(&mut c, x, 9, look.logo, WHITE, look.banner);
    }
    // Floor and apron.
    c.rect(0, 18, w, h - 18, FLOOR);
    for y in (24..h).step_by(6) {
        c.rect(0, y, w, 1, FLOOR_LINE);
    }
    c.rect(0, h - 6, w, 6, APRON);
    // Ropes behind the fighters.
    for (i, y) in [48, 54, 60].iter().enumerate() {
        let col = [Rgb::hex(0xe63946), WHITE, Rgb::hex(0x457bd9)][i];
        c.rect(0, *y, w, 2, col);
        c.rect(0, *y + 2, w, 1, col.darker(0.4));
    }
    for x in [1, w - 4] {
        c.rect(x, 40, 3, 24, Rgb::hex(0x3a3a48));
    }

    // Opponent.
    let ox = cx - 22;
    let oy = 10;
    let tell = tell_progress(&m.opp);
    let cue = m.opp.cue().is_some();
    let tint = match tell {
        Some(_) if !cue => Some((YELLOW, 0.35)),
        Some(_) => {
            if (m.tick / 2) % 2 == 0 {
                Some((RED, 0.55))
            } else {
                Some((WHITE, 0.25))
            }
        }
        None if m.opp.hit_flash > 0 => Some((WHITE, 0.6)),
        None => None,
    };
    c.set_tint(tint);
    let pose = art::opp_pose(&m.opp, m.tick, m.spec.id);
    art::draw_opponent(&mut c, ox, oy, &look, &pose, m.tick);
    c.set_tint(None);

    // Signature-move status labels, top right of the ring.
    let mut status: Vec<(String, Rgb)> = Vec::new();
    if m.cooldown > 0 {
        status.push((format!("RATE LIMITED: RETRY IN {}S", (m.cooldown + TICK_HZ - 1) / TICK_HZ), Rgb::hex(0x7fd3ff)));
    }
    if m.frozen > 0 {
        let label = if m.fought <= m.spec.prebuild + 1 { "PREBUILDING" } else { m.spec.special_name };
        status.push((format!("{}: {}S", label, (m.frozen + TICK_HZ - 1) / TICK_HZ), YELLOW));
    }
    if let Some(t) = m.notice {
        status.push((format!("NOTICE: HIT HIM IN {:.1}S", t as f32 / TICK_HZ as f32), YELLOW));
    }
    if let Some((hp, _)) = m.revoked {
        status.push((format!("HEALED +{hp:.0} HP"), RED));
    }
    if m.spec.fatigue < 1.0 {
        // The minute counter: how much of the opponent's billing is used up.
        let used = (m.fought as f32 / (3.0 * crate::sim::ROUND_TICKS as f32)).min(1.0);
        status.push((format!("CORE HOURS: {:>3.0}%", used * 100.0), Rgb::hex(0xc9d1d9)));
    }
    // Stacked up the left side of the floor, clear of both fighters.
    for (i, (s, col)) in status.iter().enumerate() {
        let sw = Canvas::text_width(s, 1);
        let x = 3;
        let y = 47 - i as i32 * 9;
        c.rect(x - 2, y - 2, sw + 4, 9, Rgb::hex(0x14141c));
        c.text(x, y, s, 1, *col);
    }

    // Tell fuse and punch name beside the head.
    if let Some((a, p)) = tell {
        let color = if cue { RED } else { YELLOW };
        // Fuse directly under the banner, where the eye already is.
        let bw = tw + 24;
        let total = m.opp.startup_ticks(a).max(1) as f32;
        // The last stretch of the track is the press zone: once the fill
        // enters it, a dodge or block pressed now catches the punch.
        let zone = ((crate::sim::CUE_LEAD as f32 / total).min(1.0) * bw as f32) as i32;
        c.rect(cx - bw / 2 - 1, 16, bw + 2, 4, Rgb::hex(0x14141c));
        c.rect(cx + bw / 2 - zone, 17, zone, 2, RED.darker(0.6));
        c.rect(cx - bw / 2, 17, (bw as f32 * p) as i32, 2, color);
        if cue {
            // Big, flashing, and on top of him: the answer to "now?".
            let word = if a == Action::Jab { "BLOCK" } else { "DODGE" };
            let ww = Canvas::text_width(word, 2);
            let col = if (m.tick / 2) % 2 == 0 { RED } else { WHITE };
            c.set_tint(None);
            c.rect(cx - ww / 2 - 3, 32, ww + 6, 14, Rgb::hex(0x14141c));
            c.text(cx - ww / 2, 34, word, 2, col);
        }
        // Punch name in both bottom corners, clear of both fighters.
        let label = match a {
            Action::Special => m.spec.special_short,
            _ => punch_name(a),
        };
        let lw = Canvas::text_width(label, 1);
        for x in [3, w - lw - 3] {
            c.rect(x - 2, 56, lw + 4, 9, Rgb::hex(0x14141c));
            c.text(x, 58, label, 1, color);
        }
    }
    // Enterprise Quote: the price ticks up beside the briefcase.
    if let (State::Startup(Action::Special, left), crate::sim::OppId::Chainguard) = (m.opp.state, m.spec.id) {
        let total = m.opp.startup_ticks(Action::Special).max(1) as f32;
        let progress = 1.0 - (left as f32 / total).clamp(0.0, 1.0);
        let price = (progress * 250_000.0) as u32;
        let s = format!("${}/YR", fmt_thousands(price));
        let sw = Canvas::text_width(&s, 1);
        let x = (ox + 50).min(w - sw - 5);
        c.rect(x - 2, 32, sw + 4, 9, Rgb::hex(0x14141c));
        c.text(x, 34, &s, 1, YELLOW);
    }

    // Player, from behind, bottom centre.
    let ppose = art::player_pose(&m.player, m.tick);
    let px = cx - 12;
    let py = h - 25;
    let ptint = if m.player.hit_flash > 0 {
        Some((RED, 0.6))
    } else if m.player.counter_window > 0 {
        Some((MAGENTA, 0.35))
    } else if matches!(m.player.state, State::Stunned(_)) {
        Some((Rgb::hex(0x14141c), 0.4))
    } else {
        None
    };
    c.set_tint(ptint);
    art::draw_player(&mut c, px, py, &art::MINIMAL, &ppose);
    c.set_tint(None);

    // Damage text beside whoever got hit. Snyk's hits come with a severity.
    for ev in &m.events {
        let (txt, color) = match ev.kind {
            HitKind::Hit if !ev.by_player && m.spec.id == crate::sim::OppId::Snyk => {
                let sev = if m.tick % 4 == 0 { "MEDIUM" } else { "CRITICAL" };
                (format!("{sev} -{:.0}", ev.damage), RED)
            }
            HitKind::Hit => (format!("-{:.0}", ev.damage), WHITE),
            HitKind::Counter => (format!("COUNTER -{:.0}", ev.damage), MAGENTA),
            HitKind::Chip => (format!("BLOCKED -{:.0}", ev.damage), Rgb::hex(0x7fd3ff)),
            HitKind::GuardBreak => (format!("GUARD BREAK -{:.0}", ev.damage), RED),
            HitKind::Interrupt => (format!("INTERRUPT -{:.0}", ev.damage), YELLOW),
            HitKind::Dodged => ("WHIFF".to_string(), Rgb::hex(0x7cf29a)),
            HitKind::Stagger => (format!("STAGGER -{:.0}", ev.damage), RED),
        };
        let (x, y) = if ev.by_player { (ox + 48, 26) } else { (px + 28, py + 4) };
        let tw = Canvas::text_width(&txt, 1);
        c.rect(x - 2, y - 2, tw + 4, 9, Rgb::hex(0x14141c));
        c.text(x, y, &txt, 1, color);
    }
    c
}

/// Text overlays on top of the pixels: debug log and the round/KO banner.
fn overlay_text(f: &mut Frame, area: Rect, m: &Match, view: &View) {
    let buf = f.buffer_mut();
    if view.show_debug {
        for (i, s) in view.last_events.iter().rev().take(6).enumerate() {
            blit_str(buf, area, area.x as i32 + 1, area.y + 1 + i as u16, s, Style::new().fg(Color::White).bg(Color::Black));
        }
    }
    if let Some((s, _)) = &m.banner {
        let (head, body, tail) = match m.phase {
            Phase::Over { player_won } => (
                Some(if player_won { "YOU WIN" } else { "YOU LOSE" }),
                s.as_str(),
                Some("[r] rematch   [enter] back to the card   [q] quit"),
            ),
            _ => (None, s.as_str(), None),
        };
        let style = Style::new().fg(Color::Black).bg(Color::Rgb(255, 210, 63)).add_modifier(Modifier::BOLD);
        let wtxt = body.chars().count().max(head.map_or(0, |h| h.len())).max(tail.map_or(0, |t| t.len())) + 4;
        let x = area.x as i32 + area.width as i32 / 2 - wtxt as i32 / 2;
        let pad = |t: &str| format!("{:^w$}", t, w = wtxt);
        let mut y = area.y + area.height / 2 - 1;
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
    let hint = match (view.show_hints, tell_progress(&m.opp)) {
        (true, Some((a, _))) => Some(match a {
            Action::Jab => "jab coming: block or dodge",
            Action::Hook => "HOOK coming: dodge",
            Action::Uppercut => "UPPERCUT coming: dodge, do not block",
            Action::Special => "SIGNATURE MOVE: DODGE IT",
        }),
        (true, None) if m.player.stamina < 20.0 && matches!(m.phase, Phase::Fighting) => {
            Some("LOW STAMINA: stop punching and let it refill, or you cannot dodge")
        }
        _ => None,
    };
    let text = if view.show_debug {
        format!("t={} p={:?} o={:?}", m.tick, m.player.state, m.opp.state)
    } else if let Some(h) = hint {
        h.to_string()
    } else {
        "[j] jab  [k] hook  [l] uppercut  [space] block  [a]/[d] dodge  [h] hints  [tab] debug  [q] quit".to_string()
    };
    let style = if hint.is_some() && !view.show_debug {
        Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::new().dim()
    };
    f.render_widget(Paragraph::new(text).style(style).alignment(Alignment::Center), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{chainguard, Input, INTRO_TICKS};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render(m: &Match, cols: u16, rows: u16) -> String {
        let view = View { show_debug: false, show_hints: true, last_events: vec![] };
        let mut t = Terminal::new(TestBackend::new(cols, rows)).unwrap();
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
    fn renders_hook_tell_at_100x40() {
        let mut m = fighting();
        m.opp.state = State::Startup(Action::Hook, 6);
        m.player_input(Input::Block);
        let out = render(&m, 100, 40);
        println!("{out}");
        assert!(out.contains("MINIMAL"));
        assert!(out.contains("HOOK coming"));
    }

    #[test]
    fn renders_special_at_120x50() {
        let mut m = fighting();
        m.opp.state = State::Startup(Action::Special, 30);
        m.player_input(Input::Jab);
        let out = render(&m, 120, 50);
        println!("{out}");
        assert!(out.contains("SIGNATURE MOVE"));
    }

    #[test]
    fn renders_ko_and_scales_up_at_200x90() {
        let mut m = fighting();
        m.opp.health = 1.0;
        m.player.state = State::Startup(Action::Hook, 1);
        m.step();
        let out = render(&m, 200, 90);
        assert!(out.contains("YOU WIN"));
    }

    #[test]
    fn small_terminal_asks_for_more_room() {
        let m = fighting();
        let out = render(&m, 80, 24);
        assert!(out.contains("enlarge"));
    }
}

#[cfg(test)]
mod frames {
    //! Dump ring frames as PNGs. Set PUNCHOUT_FRAMES=/some/dir to write them.
    use super::*;
    use crate::pixel::png;
    use crate::sim::{chainguard, Input, INTRO_TICKS};

    fn fighting() -> Match {
        let mut m = Match::new(chainguard(), 3);
        for _ in 0..(INTRO_TICKS + 40) {
            m.step();
        }
        m.banner = None;
        m
    }

    #[test]
    fn dump_frames() {
        let Ok(dir) = std::env::var("PUNCHOUT_FRAMES") else { return };
        let poses: Vec<(&str, State, Option<Input>)> = vec![
            ("idle", State::Idle, None),
            ("hook_windup", State::Startup(Action::Hook, 6), Some(Input::Block)),
            ("hook", State::Recovery(Action::Hook, 6), Some(Input::Dodge(crate::sim::Side::Left))),
            ("jab_windup", State::Startup(Action::Jab, 2), None),
            ("jab", State::Recovery(Action::Jab, 6), None),
            ("upper_windup", State::Startup(Action::Uppercut, 20), Some(Input::Jab)),
            ("upper", State::Recovery(Action::Uppercut, 10), None),
            ("special_windup", State::Startup(Action::Special, 20), Some(Input::Hook)),
            ("special", State::Recovery(Action::Special, 10), None),
            ("block", State::Blocking(6), Some(Input::Uppercut)),
            ("stunned", State::Stunned(20), None),
        ];
        for (name, st, input) in poses {
            let mut m = fighting();
            m.opp.state = st;
            if let Some(i) = input {
                m.player_input(i);
            }
            let c = paint_ring(&m, RING_W, RING_H);
            png::write(&c, &format!("{dir}/{name}.png"), 6);
        }
        // One idle frame per opponent, plus each one's special wind-up.
        for spec in crate::roster::all() {
            let n = spec.name.to_ascii_lowercase();
            let mut m = Match::new(spec, 5);
            for _ in 0..(INTRO_TICKS + 40) {
                m.step();
            }
            m.banner = None;
            m.frozen = 0;
            m.opp.state = State::Idle;
            png::write(&paint_ring(&m, RING_W, RING_H), &format!("{dir}/opp_{n}.png"), 6);
            m.opp.state = State::Startup(Action::Special, 10);
            png::write(&paint_ring(&m, RING_W, RING_H), &format!("{dir}/opp_{n}_special.png"), 6);
        }
    }
}
