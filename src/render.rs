//! Maps sim state to ratatui widgets. No game logic lives here.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Gauge, Paragraph};
use ratatui::Frame;

use crate::sim::{Action, Fighter, HitKind, Match, Phase, Side, State, TICK_HZ};

pub struct View {
    pub show_debug: bool,
    pub last_events: Vec<String>,
}

pub fn draw(f: &mut Frame, m: &Match, view: &View) {
    let area = f.area();
    let rows = Layout::vertical([
        Constraint::Length(3), // bars
        Constraint::Length(1), // round / clock
        Constraint::Min(10),   // arena
        Constraint::Length(3), // banner
        Constraint::Length(2), // help + debug
    ])
    .split(area);

    draw_bars(f, rows[0], m);
    draw_clock(f, rows[1], m);
    draw_arena(f, rows[2], m);
    draw_banner(f, rows[3], m);
    draw_footer(f, rows[4], m, view);
}

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
    let cols = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    let title = Line::from(vec![
        Span::styled(fi.name, Style::new().bold()),
        Span::raw(if fi.exhausted() { "  EXHAUSTED" } else { "" }).red().bold(),
        Span::raw(if fi.counter_window > 0 { "  COUNTER!" } else { "" }).magenta().bold(),
    ]);
    let title = if right { title.alignment(Alignment::Right) } else { title };
    f.render_widget(Paragraph::new(title), cols[0]);

    let hr = (fi.health / fi.max_health).clamp(0.0, 1.0) as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::new().fg(health_color(hr)).bg(Color::DarkGray))
            .ratio(hr)
            .label(format!("HP {:>3.0}", fi.health)),
        cols[1],
    );
    let sr = (fi.stamina / crate::sim::MAX_STAMINA).clamp(0.0, 1.0) as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::new().fg(Color::Blue).bg(Color::DarkGray))
            .ratio(sr)
            .label(format!("ST {:>3.0}", fi.stamina)),
        cols[2],
    );
}

fn draw_bars(f: &mut Frame, area: Rect, m: &Match) {
    let halves = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Percentage(10),
        Constraint::Percentage(45),
    ])
    .split(area);
    fighter_bars(f, halves[0], &m.player, false);
    fighter_bars(f, halves[2], &m.opp, true);
}

fn draw_clock(f: &mut Frame, area: Rect, m: &Match) {
    let secs = m.round_ticks_left / TICK_HZ;
    let text = format!("MINIMAL  vs  {} \"{}\"     ROUND {}/3   {:02}:{:02}", m.spec.name.to_uppercase(), m.spec.nickname, m.round, secs / 60, secs % 60);
    f.render_widget(Paragraph::new(text).alignment(Alignment::Center).bold(), area);
}

/// Prototype "sprites": boxes whose shape changes with state. Art comes later.
fn sprite(fi: &Fighter, facing_right: bool) -> Vec<String> {
    let (label, arm) = match fi.state {
        State::Idle => ("idle", "  "),
        State::Startup(a, _) => (tell_label(a), ".."),
        State::Recovery(a, _) => (action_label(a), "=="),
        State::Blocking(_) => ("BLOCK", "||"),
        State::Dodging(Side::Left, _) => ("dodge<", "  "),
        State::Dodging(Side::Right, _) => ("dodge>", "  "),
        State::Stunned(_) => ("* STUN *", "~~"),
    };
    let flash = fi.hit_flash > 0;
    let head = if flash { "(x_x)" } else if matches!(fi.state, State::Stunned(_)) { "(@_@)" } else { "(o_o)" };
    let shift = match fi.state {
        State::Dodging(Side::Left, _) => "  ",
        State::Dodging(Side::Right, _) => "      ",
        _ => "    ",
    };
    let body = if facing_right {
        vec![
            format!("{shift}  {head}  "),
            format!("{shift}  /|||\\{arm}"),
            format!("{shift} | ||| |  "),
            format!("{shift}   | |    "),
            format!("{shift}  _| |_   "),
        ]
    } else {
        vec![
            format!("{shift}  {head}  "),
            format!("{shift}{arm}/|||\\  "),
            format!("{shift} | ||| |  "),
            format!("{shift}   | |    "),
            format!("{shift}  _| |_   "),
        ]
    };
    let mut out = vec![format!("{:^14}", label)];
    out.extend(body);
    out
}

fn tell_label(a: Action) -> &'static str {
    match a {
        Action::Jab => "jab..",
        Action::Hook => "HOOK..",
        Action::Uppercut => "UPPER!!",
        Action::Special => "!!! SPECIAL !!!",
    }
}

fn action_label(a: Action) -> &'static str {
    match a {
        Action::Jab => "JAB",
        Action::Hook => "HOOK",
        Action::Uppercut => "UPPERCUT",
        Action::Special => "SPECIAL",
    }
}

fn draw_arena(f: &mut Frame, area: Rect, m: &Match) {
    let block = Block::bordered().title(" ring ");
    let inner = block.inner(area);
    f.render_widget(block, area);
    let cols = Layout::horizontal([
        Constraint::Percentage(35),
        Constraint::Percentage(30),
        Constraint::Percentage(35),
    ])
    .split(inner);

    let p_lines: Vec<Line> = sprite(&m.player, true).into_iter().map(Line::from).collect();
    let mut p_text = Text::from(p_lines);
    if m.player.hit_flash > 0 {
        p_text = p_text.style(Style::new().fg(Color::Red).add_modifier(Modifier::BOLD));
    }
    f.render_widget(Paragraph::new(p_text).alignment(Alignment::Center), vcenter(cols[0], 6));

    let o_lines: Vec<Line> = sprite(&m.opp, false).into_iter().map(Line::from).collect();
    let mut o_text = Text::from(o_lines);
    if m.opp.hit_flash > 0 {
        o_text = o_text.style(Style::new().fg(Color::Red).add_modifier(Modifier::BOLD));
    }
    f.render_widget(Paragraph::new(o_text).alignment(Alignment::Center), vcenter(cols[2], 6));

    // Middle column: special telegraph (the rolling price counter) or hit text.
    let mut mid: Vec<Line> = Vec::new();
    if let Some(t) = m.special_telegraph {
        let total = m.spec.special.startup as f32 * m.spec.speed;
        let progress = 1.0 - (t as f32 / total).clamp(0.0, 1.0);
        let price = (progress * 250_000.0) as u32;
        mid.push(Line::from(m.spec.special_name).yellow().bold());
        mid.push(Line::from(format!("${:>9}/yr", fmt_thousands(price))).yellow().bold());
        mid.push(Line::from("DODGE IT").red().bold());
    }
    for ev in &m.events {
        let who = if ev.by_player { "you" } else { m.opp.name };
        let s = match ev.kind {
            HitKind::Hit => format!("{who}: {} {:.0}", action_label(ev.action), ev.damage),
            HitKind::Counter => format!("{who}: COUNTER {:.0}!", ev.damage),
            HitKind::Chip => format!("{who}: blocked ({:.0})", ev.damage),
            HitKind::GuardBreak => format!("{who}: GUARD BREAK {:.0}", ev.damage),
            HitKind::Interrupt => format!("{who}: interrupt {:.0}", ev.damage),
            HitKind::Dodged => format!("{who}: whiff"),
            HitKind::Stagger => format!("{who}: STAGGER {:.0}", ev.damage),
        };
        mid.push(Line::from(s).bold());
    }
    f.render_widget(Paragraph::new(Text::from(mid)).alignment(Alignment::Center), vcenter(cols[1], 4));
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

fn vcenter(area: Rect, height: u16) -> Rect {
    let h = height.min(area.height);
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x: area.x, y, width: area.width, height: h }
}

fn draw_banner(f: &mut Frame, area: Rect, m: &Match) {
    let text = match (&m.phase, &m.banner) {
        (Phase::Over { player_won }, Some((s, _))) => {
            let head = if *player_won { "YOU WIN" } else { "YOU LOSE" };
            Text::from(vec![Line::from(head).bold(), Line::from(s.as_str()), Line::from("[r] rematch  [q] quit").dim()])
        }
        (_, Some((s, _))) => Text::from(Line::from(s.as_str()).bold().yellow()),
        _ => Text::default(),
    };
    f.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
}

fn draw_footer(f: &mut Frame, area: Rect, m: &Match, view: &View) {
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);
    let help = "[j] jab  [k] hook  [l] uppercut  [space] block  [a]/[d] dodge  [tab] debug  [q] quit";
    f.render_widget(Paragraph::new(help).dim().alignment(Alignment::Center), rows[0]);
    if view.show_debug {
        let dbg = format!(
            "t={} p={:?} o={:?} seed-run | {}",
            m.tick,
            m.player.state,
            m.opp.state,
            view.last_events.join(" | ")
        );
        f.render_widget(Paragraph::new(dbg).dim(), rows[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{chainguard, Input};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_a_frame_at_80x24() {
        let mut m = Match::new(chainguard(), 3);
        for _ in 0..(crate::sim::INTRO_TICKS + 5) {
            m.step();
        }
        m.player_input(Input::Jab);
        m.step();
        m.step();
        m.opp.state = State::Startup(Action::Special, 20);
        m.special_telegraph = Some(20);
        let view = View { show_debug: true, last_events: vec!["x".into()] };
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| draw(f, &m, &view)).unwrap();
        let buf = t.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        println!("{out}");
        assert!(out.contains("Minimal"));
        assert!(out.contains("Chainguard"));
    }
}
