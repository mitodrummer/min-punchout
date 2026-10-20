mod art;
mod pixel;
mod render;
mod roster;
mod sim;

use std::io::Write;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use sim::{Input, Match, Phase, Side, TICK_HZ};

struct Opts {
    seed: u64,
    bell: bool,
    rank: Option<u8>,
    opponent: Option<String>,
}

fn parse_opts() -> Opts {
    let mut opts = Opts {
        seed: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(1),
        bell: true,
        rank: None,
        opponent: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seed" => {
                if let Some(v) = args.next().and_then(|s| s.parse().ok()) {
                    opts.seed = v;
                }
            }
            "--quiet" | "-q" => opts.bell = false,
            "--rank" => {
                if let Some(v) = args.next().and_then(|s| s.parse().ok()) {
                    opts.rank = Some(v);
                }
            }
            "--opponent" | "-o" => {
                let name = args.next().unwrap_or_default();
                if roster::by_name(&name).is_none() {
                    eprintln!("unknown opponent '{name}'. The card, easiest first:");
                    for s in roster::all() {
                        eprintln!("  {}. {} \"{}\"", s.rank, s.name, s.nickname);
                    }
                    std::process::exit(2);
                }
                opts.opponent = Some(name);
            }
            "-h" | "--help" => {
                println!("punchout [--opponent NAME] [--rank 1-8] [--seed N] [--quiet]");
                std::process::exit(0);
            }
            _ => {}
        }
    }
    opts
}

fn bell(enabled: bool) {
    if enabled {
        let mut out = std::io::stdout();
        let _ = out.write_all(b"\x07");
        let _ = out.flush();
    }
}

fn main() -> std::io::Result<()> {
    let opts = parse_opts();
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &opts);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, opts: &Opts) -> std::io::Result<()> {
    let tick = Duration::from_micros(1_000_000 / TICK_HZ as u64);
    // Opponent: named on the command line, else random for this seed. The
    // rank dial, if given, re-tunes whoever was picked.
    let pick = |seed: u64, named: bool| -> sim::OpponentSpec {
        let base = match (&opts.opponent, named) {
            (Some(n), true) => roster::by_name(n).expect("validated in parse_opts"),
            _ => {
                let all = roster::all();
                all[(seed % all.len() as u64) as usize].clone()
            }
        };
        match opts.rank {
            Some(r) => base.at_rank(r),
            None => base,
        }
    };
    let mut m = Match::new(pick(opts.seed, true), opts.seed);
    let mut view = render::View { show_debug: false, show_hints: true, last_events: Vec::new() };
    let mut next = Instant::now() + tick;

    loop {
        // Drain input until the tick deadline.
        loop {
            let now = Instant::now();
            if now >= next {
                break;
            }
            if event::poll(next - now)? {
                if let Event::Key(k) = event::read()? {
                    // Key repeat from a held key must not spam actions.
                    if k.kind != KeyEventKind::Press {
                        continue;
                    }
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Tab => view.show_debug = !view.show_debug,
                        KeyCode::Char('h') => view.show_hints = !view.show_hints,
                        KeyCode::Char('r') => {
                            if matches!(m.phase, Phase::Over { .. }) {
                                let seed = opts.seed.wrapping_add(m.tick);
                                m = Match::new(pick(opts.seed, true), seed);
                            }
                        }
                        KeyCode::Char('n') => {
                            if matches!(m.phase, Phase::Over { .. }) {
                                let seed = opts.seed.wrapping_add(m.tick).wrapping_mul(2654435761);
                                m = Match::new(pick(seed, false), seed);
                            }
                        }
                        KeyCode::Char('j') => {
                            m.player_input(Input::Jab);
                        }
                        KeyCode::Char('k') => {
                            m.player_input(Input::Hook);
                        }
                        KeyCode::Char('l') => {
                            m.player_input(Input::Uppercut);
                        }
                        KeyCode::Char(' ') => {
                            m.player_input(Input::Block);
                        }
                        KeyCode::Char('a') | KeyCode::Left => {
                            m.player_input(Input::Dodge(Side::Left));
                        }
                        KeyCode::Char('d') | KeyCode::Right => {
                            m.player_input(Input::Dodge(Side::Right));
                        }
                        _ => {}
                    }
                }
            }
        }
        next += tick;
        // If we fell far behind (suspended terminal), resync instead of spiralling.
        if Instant::now() > next + tick * 5 {
            next = Instant::now() + tick;
        }

        let ring = m.step();
        if ring {
            bell(opts.bell);
            if matches!(m.phase, Phase::Over { .. }) {
                bell(opts.bell);
                bell(opts.bell);
            }
        }
        if !m.events.is_empty() {
            for ev in &m.events {
                view.last_events.push(format!("{:?} {:?} {:.0}", ev.kind, ev.action, ev.damage));
            }
            let n = view.last_events.len();
            if n > 4 {
                view.last_events.drain(0..n - 4);
            }
        }
        terminal.draw(|f| render::draw(f, &m, &view))?;
    }
}
