mod art;
mod board;
mod pixel;
mod render;
mod roster;
mod sim;
mod sound;

use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use board::Board;
use sim::{Action, HitEvent, HitKind, Input, Match, OpponentSpec, Phase, Side, State, TICK_HZ};
use sound::{Audio, Music, Sfx};

struct Opts {
    seed: u64,
    bell: bool,
    rank: Option<u8>,
    opponent: Option<String>,
    log: Option<String>,
    no_music: bool,
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
        log: None,
        no_music: false,
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
            "--no-music" => opts.no_music = true,
            "--log" => opts.log = Some(args.next().unwrap_or_else(|| "punchout.log".into())),
            "-h" | "--help" => {
                println!("punchout [--opponent NAME] [--rank 1-8] [--seed N] [--quiet] [--no-music] [--log FILE]");
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

/// Session log for tuning: every key press with where the opponent was in
/// his wind-up, and every hit that resolved. One line per record.
struct Log(Option<File>);

impl Log {
    fn open(path: &Option<String>) -> Log {
        Log(path.as_ref().and_then(|p| File::create(p).ok()))
    }

    fn line(&mut self, s: String) {
        if let Some(f) = &mut self.0 {
            let _ = writeln!(f, "{s}");
        }
    }

    fn fight(&mut self, spec: &OpponentSpec, seed: u64) {
        self.line(format!("fight opp={} rank={} seed={seed}", spec.name, spec.rank));
    }

    /// A key the player pressed. `left` is how many ticks before the
    /// opponent's punch would land, when he is winding one up.
    fn key(&mut self, m: &Match, key: &str, accepted: bool) {
        let (opp, left) = match m.opp.state {
            State::Startup(a, left) => (format!("windup:{a:?}"), left.to_string()),
            ref s => (format!("{s:?}").to_lowercase(), "-".into()),
        };
        let opp = opp.split('(').next().unwrap_or("").to_string();
        let ok = if accepted { "ok" } else { "ignored" };
        self.line(format!(
            "key t={} {key} {ok} opp={opp} left={left} player={} stam={:.0}",
            m.tick,
            format!("{:?}", m.player.state).split('(').next().unwrap_or("").to_lowercase(),
            m.player.stamina
        ));
    }

    fn events(&mut self, m: &Match) {
        for ev in &m.events {
            let who = if ev.by_player { "player" } else { "opp" };
            self.line(format!("hit t={} by={who} {:?} {:?} dmg={:.0} php={:.0} ohp={:.0}", m.tick, ev.action, ev.kind, ev.damage, m.player.health, m.opp.health));
        }
    }

    fn over(&mut self, m: &Match, won: bool) {
        self.line(format!("over won={won} t={} round={} php={:.0} ohp={:.0}", m.tick, m.round, m.player.health, m.opp.health));
    }
}

enum Screen {
    Card,
    Fight(Match, usize),
}

fn play(audio: &Option<Audio>, s: Sfx) {
    if let Some(a) = audio {
        a.play(s);
    }
}

fn sfx_for(ev: &HitEvent) -> Sfx {
    use HitKind::*;
    match (ev.by_player, ev.kind) {
        (true, Hit) | (true, Stagger) => {
            if ev.action == Action::Jab { Sfx::Jab } else { Sfx::Heavy }
        }
        (true, Counter) => Sfx::Counter,
        (true, GuardBreak) => Sfx::Heavy,
        (_, Chip) | (_, Interrupt) => Sfx::Block,
        (_, Dodged) => Sfx::Whoosh,
        (false, _) => Sfx::Hurt,
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
    let specs = roster::all();
    let mut card = Board::new(specs.len());
    if let Some(n) = &opts.opponent {
        let named = roster::by_name(n).expect("validated in parse_opts");
        if let Some(i) = specs.iter().position(|s| s.id == named.id) {
            card.unlock_to(i);
        }
    }
    // The rank dial, if given, re-tunes whoever is picked.
    let spec_for = |i: usize| -> OpponentSpec {
        let base = specs[i].clone();
        match opts.rank {
            Some(r) => base.at_rank(r),
            None => base,
        }
    };
    let mut log = Log::open(&opts.log);
    // --quiet silences everything. Without an audio device we fall back to
    // the terminal bell.
    let mut audio = if opts.bell { Audio::new(!opts.no_music) } else { None };
    let mut screen = Screen::Card;
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
                        _ => {}
                    }
                    match &mut screen {
                        Screen::Card => match k.code {
                            KeyCode::Char('a') | KeyCode::Left | KeyCode::Up => {
                                card.move_cursor(-1);
                                play(&audio, Sfx::Menu);
                            }
                            KeyCode::Char('d') | KeyCode::Right | KeyCode::Down => {
                                card.move_cursor(1);
                                play(&audio, Sfx::Menu);
                            }
                            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Char('l') => {
                                let seed = opts.seed.wrapping_add(card.tick);
                                let idx = card.cursor;
                                let spec = spec_for(idx);
                                log.fight(&spec, seed);
                                play(&audio, Sfx::Confirm);
                                if let Some(a) = &mut audio {
                                    a.music(Music::Fight);
                                }
                                screen = Screen::Fight(Match::new(spec, seed), idx);
                            }
                            _ => {}
                        },
                        Screen::Fight(m, idx) => {
                            let idx = *idx;
                            let over = matches!(m.phase, Phase::Over { .. });
                            let input = match k.code {
                                KeyCode::Char('j') => Some((Input::Jab, "jab")),
                                KeyCode::Char('k') => Some((Input::Hook, "hook")),
                                KeyCode::Char('l') => Some((Input::Uppercut, "uppercut")),
                                KeyCode::Char(' ') => Some((Input::Block, "block")),
                                KeyCode::Char('a') | KeyCode::Left => Some((Input::Dodge(Side::Left), "dodge")),
                                KeyCode::Char('d') | KeyCode::Right => Some((Input::Dodge(Side::Right), "dodge")),
                                _ => None,
                            };
                            if let (Some((i, name)), false) = (input, over) {
                                let accepted = m.player_input(i);
                                log.key(m, name, accepted);
                            }
                            if over {
                                match k.code {
                                    KeyCode::Char('r') => {
                                        let seed = opts.seed.wrapping_add(m.tick).wrapping_add(card.tick);
                                        let spec = spec_for(idx);
                                        log.fight(&spec, seed);
                                        if let Some(a) = &mut audio {
                                            a.music(Music::Fight);
                                        }
                                        screen = Screen::Fight(Match::new(spec, seed), idx);
                                    }
                                    KeyCode::Enter | KeyCode::Char('n') => screen = Screen::Card,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        next += tick;
        // If we fell far behind (suspended terminal), resync instead of spiralling.
        if Instant::now() > next + tick * 5 {
            next = Instant::now() + tick;
        }

        card.tick += 1;
        match &mut screen {
            Screen::Card => {
                if let Some(a) = &mut audio {
                    a.music(Music::Card);
                }
                terminal.draw(|f| board::draw(f, &card, &specs))?;
            }
            Screen::Fight(m, idx) => {
                let was_over = matches!(m.phase, Phase::Over { .. });
                let cue_before = m.opp.cue();
                let stunned_before = matches!(m.opp.state, State::Stunned(_)) || matches!(m.player.state, State::Stunned(_));
                let special_before = matches!(m.opp.state, State::Startup(Action::Special, _));
                let ring = m.step();
                if ring {
                    match &audio {
                        Some(a) => a.play(Sfx::Bell),
                        None => bell(opts.bell),
                    }
                    if let Phase::Over { player_won } = m.phase {
                        if !was_over {
                            log.over(m, player_won);
                            if player_won {
                                card.beat(*idx);
                            }
                            if let Some(a) = &mut audio {
                                a.music(Music::Off);
                                a.play(if player_won { Sfx::Win } else { Sfx::Lose });
                            } else {
                                bell(opts.bell);
                                bell(opts.bell);
                            }
                        }
                    }
                }
                if let Some(a) = &audio {
                    if cue_before.is_none() {
                        if let Some(act) = m.opp.cue() {
                            a.play(if act == Action::Jab { Sfx::CueJab } else { Sfx::Cue });
                        }
                    }
                    if !special_before && matches!(m.opp.state, State::Startup(Action::Special, _)) {
                        a.play(Sfx::Alarm);
                    }
                    let stunned_now = matches!(m.opp.state, State::Stunned(_)) || matches!(m.player.state, State::Stunned(_));
                    if !stunned_before && stunned_now {
                        a.play(Sfx::Stun);
                    }
                    for ev in &m.events {
                        a.play(sfx_for(ev));
                    }
                }
                if !m.events.is_empty() {
                    log.events(m);
                    for ev in &m.events {
                        view.last_events.push(format!("{:?} {:?} {:.0}", ev.kind, ev.action, ev.damage));
                    }
                    let n = view.last_events.len();
                    if n > 4 {
                        view.last_events.drain(0..n - 4);
                    }
                }
                terminal.draw(|f| render::draw(f, m, &view))?;
            }
        }
    }
}
