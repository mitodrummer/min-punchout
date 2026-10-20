//! Pure match simulation. No terminal dependency; everything is in ticks.
//! TICK_HZ ticks per second. Timings from the design doc converted to ticks.

pub const TICK_HZ: u32 = 30;
pub const ROUND_TICKS: u32 = 60 * TICK_HZ;
pub const ROUNDS: u8 = 3;
pub const MAX_HEALTH: f32 = 100.0;
pub const MAX_STAMINA: f32 = 100.0;
/// Stamina per second regained while idle.
pub const STAMINA_REGEN_PER_SEC: f32 = 22.0;
/// Below this you cannot block or dodge and any hit staggers you.
pub const EXHAUSTED_BELOW: f32 = 6.0;
pub const COUNTER_WINDOW: u32 = 12;
pub const CHIP_FRACTION: f32 = 0.15;
/// Ticks you must stay idle before stamina starts coming back. The anti-mash rule.
pub const REGEN_DELAY: u32 = 10;
/// Regen multiplier while winded, until stamina climbs back above EXHAUSTED_RECOVER.
pub const EXHAUSTED_REGEN: f32 = 0.5;
pub const EXHAUSTED_RECOVER: f32 = 25.0;
pub const GUARD_BREAK_STUN: u32 = 24;
pub const EXHAUSTED_STUN: u32 = 30;
pub const INTERRUPT_STUN: u32 = 8;

fn ms(ms: u32) -> u32 {
    ((ms * TICK_HZ) as f32 / 1000.0).round().max(1.0) as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Jab,
    Hook,
    Uppercut,
    /// Opponent signature move. Parameters live on the opponent spec.
    Special,
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub stamina: f32,
    pub damage: f32,
    pub startup: u32,
    pub recovery: u32,
}

pub const JAB: Move = Move { stamina: 8.0, damage: 6.0, startup: 3, recovery: 6 };
pub const HOOK: Move = Move { stamina: 14.0, damage: 12.0, startup: 8, recovery: 10 };
pub const UPPERCUT: Move = Move { stamina: 25.0, damage: 22.0, startup: 15, recovery: 18 };
pub const BLOCK_COST: f32 = 6.0;
pub const BLOCK_WINDOW: u32 = 12;
pub const DODGE_COST: f32 = 10.0;
pub const DODGE_WINDOW: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Startup(Action, u32),
    Recovery(Action, u32),
    Blocking(u32),
    Dodging(Side, u32),
    Stunned(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Jab,
    Hook,
    Uppercut,
    Block,
    Dodge(Side),
    Special,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub name: &'static str,
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub regen_per_sec: f32,
    /// Multiplies startup and recovery. >1 is slower.
    pub speed: f32,
    /// Extra wind-up ticks added to every punch. Gives opponents a readable tell.
    pub tell_bonus: u32,
    pub state: State,
    pub counter_window: u32,
    /// Consecutive ticks spent in Idle.
    pub idle_ticks: u32,
    /// Set when stamina hits the floor; cleared once it climbs back.
    pub winded: bool,
    /// Ticks since last hit landed on this fighter, for render flashes.
    pub hit_flash: u32,
    pub special: Option<Move>,
}

impl Fighter {
    pub fn new(name: &'static str) -> Self {
        Fighter {
            name,
            health: MAX_HEALTH,
            max_health: MAX_HEALTH,
            stamina: MAX_STAMINA,
            regen_per_sec: STAMINA_REGEN_PER_SEC,
            speed: 1.0,
            tell_bonus: 0,
            state: State::Idle,
            counter_window: 0,
            idle_ticks: 0,
            winded: false,
            hit_flash: 0,
            special: None,
        }
    }

    pub fn exhausted(&self) -> bool {
        self.stamina < EXHAUSTED_BELOW
    }

    pub fn can_act(&self) -> bool {
        matches!(self.state, State::Idle)
    }

    fn scaled(&self, t: u32) -> u32 {
        ((t as f32) * self.speed).round().max(1.0) as u32
    }

    /// Full wind-up length for a punch, including this fighter's tell bonus.
    pub fn startup_ticks(&self, a: Action) -> u32 {
        // Jabs stay quick: they are the punch you block, not the one you read.
        let bonus = if a == Action::Jab { 0 } else { self.tell_bonus };
        self.scaled(self.mv(a).startup) + bonus
    }

    pub fn mv(&self, a: Action) -> Move {
        match a {
            Action::Jab => JAB,
            Action::Hook => HOOK,
            Action::Uppercut => UPPERCUT,
            Action::Special => self.special.unwrap_or(UPPERCUT),
        }
    }

    /// Try to start an input. Returns false if ignored (busy or no stamina).
    pub fn apply(&mut self, input: Input) -> bool {
        if !self.can_act() {
            return false;
        }
        match input {
            Input::Jab | Input::Hook | Input::Uppercut | Input::Special => {
                let a = match input {
                    Input::Jab => Action::Jab,
                    Input::Hook => Action::Hook,
                    Input::Uppercut => Action::Uppercut,
                    _ => Action::Special,
                };
                let m = self.mv(a);
                if self.stamina < m.stamina {
                    return false;
                }
                self.stamina -= m.stamina;
                self.state = State::Startup(a, self.startup_ticks(a));
                true
            }
            Input::Block => {
                if self.exhausted() || self.stamina < BLOCK_COST {
                    return false;
                }
                self.stamina -= BLOCK_COST;
                self.state = State::Blocking(BLOCK_WINDOW);
                true
            }
            Input::Dodge(side) => {
                if self.exhausted() || self.stamina < DODGE_COST {
                    return false;
                }
                self.stamina -= DODGE_COST;
                self.state = State::Dodging(side, DODGE_WINDOW);
                true
            }
        }
    }

    /// Advance timers by one tick. Returns Some(action) on the tick a punch lands.
    fn tick(&mut self) -> Option<Action> {
        if self.counter_window > 0 {
            self.counter_window -= 1;
        }
        if self.hit_flash > 0 {
            self.hit_flash -= 1;
        }
        if self.stamina < EXHAUSTED_BELOW {
            self.winded = true;
        } else if self.stamina >= EXHAUSTED_RECOVER {
            self.winded = false;
        }
        // Only attacking (and getting stunned) resets the regen clock. Blocking
        // and dodging pause regen but do not restart the delay, so a guard is
        // sustainable while a flurry of punches is not.
        if matches!(self.state, State::Startup(..) | State::Recovery(..) | State::Stunned(_)) {
            self.idle_ticks = 0;
        }
        match self.state {
            State::Idle => {
                self.idle_ticks += 1;
                if self.idle_ticks > REGEN_DELAY {
                    let mut r = self.regen_per_sec / TICK_HZ as f32;
                    if self.winded {
                        r *= EXHAUSTED_REGEN;
                    }
                    self.stamina = (self.stamina + r).min(MAX_STAMINA);
                }
                None
            }
            State::Startup(a, t) => {
                if t <= 1 {
                    let m = self.mv(a);
                    self.state = State::Recovery(a, self.scaled(m.recovery));
                    Some(a)
                } else {
                    self.state = State::Startup(a, t - 1);
                    None
                }
            }
            State::Recovery(a, t) => {
                self.state = if t <= 1 { State::Idle } else { State::Recovery(a, t - 1) };
                None
            }
            State::Blocking(t) => {
                self.state = if t <= 1 { State::Idle } else { State::Blocking(t - 1) };
                None
            }
            State::Dodging(s, t) => {
                self.state = if t <= 1 { State::Idle } else { State::Dodging(s, t - 1) };
                None
            }
            State::Stunned(t) => {
                // You catch your breath while stunned. Without this a winded
                // fighter gets staggered again the instant the stun ends and the
                // fight degenerates into a stun lock.
                let r = self.regen_per_sec / TICK_HZ as f32 * 1.5;
                self.stamina = (self.stamina + r).min(MAX_STAMINA);
                self.state = if t <= 1 { State::Idle } else { State::Stunned(t - 1) };
                None
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitKind {
    Hit,
    Counter,
    Chip,
    GuardBreak,
    Interrupt,
    Dodged,
    Stagger,
}

#[derive(Clone, Copy, Debug)]
pub struct HitEvent {
    /// true when the player is the attacker.
    pub by_player: bool,
    pub action: Action,
    pub kind: HitKind,
    pub damage: f32,
}

/// Resolve `action` from `att` landing on `def`.
fn resolve(att: &mut Fighter, def: &mut Fighter, action: Action, by_player: bool) -> HitEvent {
    let base = att.mv(action).damage;
    let mut ev = HitEvent { by_player, action, kind: HitKind::Hit, damage: 0.0 };
    match def.state {
        State::Dodging(..) => {
            ev.kind = HitKind::Dodged;
            def.counter_window = COUNTER_WINDOW;
            return ev;
        }
        State::Blocking(..) => {
            if matches!(action, Action::Uppercut | Action::Special) {
                ev.kind = HitKind::GuardBreak;
                ev.damage = base;
                def.state = State::Stunned(GUARD_BREAK_STUN);
            } else {
                ev.kind = HitKind::Chip;
                ev.damage = base * CHIP_FRACTION;
            }
        }
        // A jab interrupts a heavy punch only in the first half of its wind-up.
        // Late in the swing the heavy punch is committed and lands regardless.
        State::Startup(heavy @ (Action::Uppercut | Action::Special), left)
            if action == Action::Jab && left * 2 > def.startup_ticks(heavy) =>
        {
            ev.kind = HitKind::Interrupt;
            ev.damage = base;
            def.state = State::Stunned(INTERRUPT_STUN);
        }
        _ => {
            ev.damage = base;
            if def.exhausted() && !matches!(def.state, State::Stunned(_)) {
                ev.kind = HitKind::Stagger;
                def.state = State::Stunned(EXHAUSTED_STUN);
            }
        }
    }
    if att.counter_window > 0 && ev.kind != HitKind::Chip {
        ev.damage *= 2.0;
        ev.kind = HitKind::Counter;
        att.counter_window = 0;
    }
    def.health = (def.health - ev.damage).max(0.0);
    def.hit_flash = 4;
    ev
}

/// Tiny seedable xorshift64*. Deterministic replays for tuning.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[(self.next_u64() % xs.len() as u64) as usize]
    }
}

/// A step in an opponent pattern.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    Do(Input),
    Wait(u32),
}

#[derive(Clone, Debug)]
pub struct OpponentSpec {
    pub name: &'static str,
    pub nickname: &'static str,
    pub health: f32,
    pub regen_per_sec: f32,
    pub speed: f32,
    pub tell_bonus: u32,
    pub block_chance: f32,
    pub dodge_chance: f32,
    pub reaction_delay: u32,
    pub patterns: Vec<Vec<Step>>,
    pub special: Move,
    /// Ticks between special attempts.
    pub special_every: u32,
    pub special_name: &'static str,
    pub special_stun_on_dodge: u32,
    pub taunts: [&'static str; 3],
    pub ko_line: &'static str,
    pub lose_line: &'static str,
}

pub fn chainguard() -> OpponentSpec {
    OpponentSpec {
        name: "Chainguard",
        nickname: "The Zero Day",
        health: 120.0,
        regen_per_sec: 24.0,
        speed: 1.5,
        tell_bonus: 8,
        block_chance: 0.9,
        dodge_chance: 0.05,
        reaction_delay: 2,
        patterns: vec![
            vec![Step::Do(Input::Hook), Step::Wait(ms(500))],
            vec![Step::Do(Input::Jab), Step::Do(Input::Jab), Step::Wait(ms(400))],
            vec![Step::Do(Input::Block), Step::Wait(ms(300))],
            vec![Step::Wait(ms(350))],
        ],
        special: Move { stamina: 30.0, damage: 30.0, startup: 45, recovery: 20 },
        special_every: 15 * TICK_HZ,
        special_name: "ENTERPRISE QUOTE",
        special_stun_on_dodge: 60,
        taunts: [
            "Zero CVEs. Zero free tier. Zero chance.",
            "Our guard is SLSA level 3.",
            "Have you considered a multi-year commitment?",
        ],
        ko_line: "Would you like to schedule a call to discuss your defeat?",
        lose_line: "We'll follow up with a revised quote.",
    }
}

#[derive(Clone, Debug)]
struct Brain {
    queue: Vec<Step>,
    wait: u32,
    pending: Option<(Input, u32)>,
    since_special: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Pre-round banner and taunt.
    Intro(u32),
    Fighting,
    /// Between rounds.
    Break(u32),
    Over { player_won: bool },
}

#[derive(Clone, Debug)]
pub struct Match {
    pub player: Fighter,
    pub opp: Fighter,
    pub spec: OpponentSpec,
    pub round: u8,
    pub round_ticks_left: u32,
    pub phase: Phase,
    pub tick: u64,
    pub events: Vec<HitEvent>,
    pub banner: Option<(String, u32)>,
    pub special_telegraph: Option<u32>,
    pub rng: Rng,
    brain: Brain,
}

pub const INTRO_TICKS: u32 = 3 * TICK_HZ;
pub const BREAK_TICKS: u32 = 4 * TICK_HZ;

impl Match {
    pub fn new(spec: OpponentSpec, seed: u64) -> Self {
        let mut opp = Fighter::new(spec.name);
        opp.health = spec.health;
        opp.max_health = spec.health;
        opp.regen_per_sec = spec.regen_per_sec;
        opp.speed = spec.speed;
        opp.tell_bonus = spec.tell_bonus;
        opp.special = Some(spec.special);
        Match {
            player: Fighter::new("Minimal"),
            opp,
            spec,
            round: 1,
            round_ticks_left: ROUND_TICKS,
            phase: Phase::Intro(INTRO_TICKS),
            tick: 0,
            events: Vec::new(),
            banner: None,
            special_telegraph: None,
            rng: Rng::new(seed),
            brain: Brain { queue: Vec::new(), wait: ms(800), pending: None, since_special: 0 },
        }
    }

    pub fn player_input(&mut self, input: Input) -> bool {
        if self.phase != Phase::Fighting {
            return false;
        }
        self.player.apply(input)
    }

    fn say(&mut self, s: impl Into<String>, ticks: u32) {
        self.banner = Some((s.into(), ticks));
    }

    /// One tick. Returns true when a bell should ring.
    pub fn step(&mut self) -> bool {
        self.tick += 1;
        self.events.clear();
        if let Some((_, t)) = &mut self.banner {
            if *t <= 1 {
                self.banner = None;
            } else {
                *t -= 1;
            }
        }
        match self.phase {
            Phase::Intro(t) => {
                if t == INTRO_TICKS {
                    let taunt = self.spec.taunts[(self.round as usize - 1) % 3];
                    self.say(format!("ROUND {}  -  {}", self.round, taunt), INTRO_TICKS);
                }
                if t <= 1 {
                    self.phase = Phase::Fighting;
                    self.say("FIGHT", TICK_HZ);
                    return true;
                }
                self.phase = Phase::Intro(t - 1);
                false
            }
            Phase::Break(t) => {
                if t <= 1 {
                    self.round += 1;
                    self.round_ticks_left = ROUND_TICKS;
                    self.player.stamina = MAX_STAMINA;
                    self.opp.stamina = MAX_STAMINA;
                    self.player.state = State::Idle;
                    self.opp.state = State::Idle;
                    self.player.counter_window = 0;
                    self.opp.counter_window = 0;
                    self.phase = Phase::Intro(INTRO_TICKS);
                } else {
                    self.phase = Phase::Break(t - 1);
                }
                false
            }
            Phase::Over { .. } => false,
            Phase::Fighting => self.step_fight(),
        }
    }

    fn step_fight(&mut self) -> bool {
        self.think();
        let p_hit = self.player.tick();
        let o_hit = self.opp.tick();
        if let Some(a) = p_hit {
            let ev = resolve(&mut self.player, &mut self.opp, a, true);
            if ev.kind == HitKind::Chip && a == Action::Hook {
                // A blocked hook leaves a long enough recovery to punish with a
                // jab. A blocked jab does not: answering it only trades damage.
                self.brain.pending = Some((Input::Jab, 1));
            }
            self.events.push(ev);
        }
        if let Some(a) = o_hit {
            let ev = resolve(&mut self.opp, &mut self.player, a, false);
            if a == Action::Special && ev.kind == HitKind::Dodged {
                self.opp.state = State::Stunned(self.spec.special_stun_on_dodge);
                self.say("QUOTE DECLINED", TICK_HZ);
            }
            self.events.push(ev);
        }
        self.special_telegraph = match self.opp.state {
            State::Startup(Action::Special, t) => Some(t),
            _ => None,
        };

        if self.opp.health <= 0.0 {
            self.phase = Phase::Over { player_won: true };
            self.say(format!("KO!  {}", self.spec.ko_line), u32::MAX);
            return true;
        }
        if self.player.health <= 0.0 {
            self.phase = Phase::Over { player_won: false };
            self.say(format!("KO!  {}", self.spec.lose_line), u32::MAX);
            return true;
        }
        self.round_ticks_left -= 1;
        if self.round_ticks_left == 0 {
            if self.round >= ROUNDS {
                // Ties go to Minimal. It is our game.
                let won = self.player.health >= self.opp.health;
                self.phase = Phase::Over { player_won: won };
                let line = if won { "DECISION: MINIMAL" } else { "DECISION: CHAINGUARD" };
                self.say(line, u32::MAX);
            } else {
                self.phase = Phase::Break(BREAK_TICKS);
                self.say(format!("END OF ROUND {}", self.round), BREAK_TICKS);
            }
            return true;
        }
        false
    }

    fn think(&mut self) {
        let b = &mut self.brain;
        b.since_special += 1;
        if !self.opp.can_act() {
            return;
        }
        // Scheduled reaction fires first.
        // A reaction scheduled with delay N fires on the Nth think after it
        // was scheduled, so a 2-tick reaction to a 3-tick jab lands the block
        // before the jab does.
        if let Some((input, delay)) = b.pending {
            if delay <= 1 {
                b.pending = None;
                self.opp.apply(input);
                return;
            }
            b.pending = Some((input, delay - 1));
            return;
        }
        // Punish: a recovering, stunned or exhausted player is an open target.
        // Only openings long enough for a hook to actually land count. A jab's
        // recovery is too short; an uppercut's is not, which is what makes the
        // uppercut risky.
        let hook_lands_in = self.opp.scaled(HOOK.startup) + 1;
        let open = match self.player.state {
            State::Recovery(_, t) => t >= hook_lands_in,
            State::Stunned(t) => t >= hook_lands_in,
            // Never start a punish while a punch is already on its way in;
            // that is what blocking is for.
            State::Startup(..) => false,
            _ => self.player.exhausted(),
        };
        if open {
            // A stunned player eats the uppercut. An exhausted one who can still
            // twitch a jab gets the faster hook, which cannot be interrupted.
            let heavy = matches!(self.player.state, State::Stunned(_));
            let (input, action) = if heavy && self.opp.stamina >= UPPERCUT.stamina {
                (Input::Uppercut, Action::Uppercut)
            } else {
                (Input::Hook, Action::Hook)
            };
            if self.opp.stamina >= self.opp.mv(action).stamina {
                b.pending = Some((input, self.spec.reaction_delay));
                return;
            }
        }
        // React to a telegraphed player attack.
        if let State::Startup(a, _) = self.player.state {
            let roll = self.rng.unit();
            let react = if roll < self.spec.block_chance {
                Some(Input::Block)
            } else if roll < self.spec.block_chance + self.spec.dodge_chance {
                Some(Input::Dodge(Side::Left))
            } else {
                None
            };
            // Uppercuts are slow enough that even a blocker sometimes dodges.
            let react = if a == Action::Uppercut && self.rng.chance(0.25) {
                Some(Input::Dodge(Side::Right))
            } else {
                react
            };
            if let Some(r) = react {
                b.pending = Some((r, self.spec.reaction_delay));
                return;
            }
        }
        if b.wait > 0 {
            b.wait -= 1;
            return;
        }
        // A blocker does not trade with a tiring masher. Keep the guard up and
        // let them run dry; the punish rule above takes over once they do.
        if self.player.stamina < 40.0 && !self.player.exhausted() && self.spec.block_chance >= 0.75 {
            b.wait = 3;
            return;
        }
        if b.since_special >= self.spec.special_every && self.opp.stamina >= self.spec.special.stamina {
            b.since_special = 0;
            self.opp.apply(Input::Special);
            self.say(self.spec.special_name, 45);
            return;
        }
        if b.queue.is_empty() {
            let pat = self.rng.pick(&self.spec.patterns).clone();
            b.queue = pat;
        }
        let step = b.queue.remove(0);
        match step {
            Step::Wait(t) => b.wait = t,
            Step::Do(input) => {
                if !self.opp.apply(input) {
                    // Not enough stamina: rest instead.
                    b.wait = ms(500);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Fighter, Fighter) {
        (Fighter::new("a"), Fighter::new("b"))
    }

    #[test]
    fn jab_lands_on_idle() {
        let (mut a, mut d) = pair();
        let ev = resolve(&mut a, &mut d, Action::Jab, true);
        assert_eq!(ev.kind, HitKind::Hit);
        assert_eq!(d.health, MAX_HEALTH - JAB.damage);
    }

    #[test]
    fn block_chips_jab_but_uppercut_breaks_guard() {
        let (mut a, mut d) = pair();
        d.state = State::Blocking(5);
        let ev = resolve(&mut a, &mut d, Action::Jab, true);
        assert_eq!(ev.kind, HitKind::Chip);
        assert!((ev.damage - JAB.damage * CHIP_FRACTION).abs() < 1e-5);
        d.state = State::Blocking(5);
        let ev = resolve(&mut a, &mut d, Action::Uppercut, true);
        assert_eq!(ev.kind, HitKind::GuardBreak);
        assert_eq!(d.state, State::Stunned(GUARD_BREAK_STUN));
    }

    #[test]
    fn dodge_opens_counter_and_counter_doubles() {
        let (mut a, mut d) = pair();
        d.state = State::Dodging(Side::Left, 3);
        let ev = resolve(&mut a, &mut d, Action::Hook, false);
        assert_eq!(ev.kind, HitKind::Dodged);
        assert_eq!(d.counter_window, COUNTER_WINDOW);
        d.state = State::Idle;
        let ev = resolve(&mut d, &mut a, Action::Hook, true);
        assert_eq!(ev.kind, HitKind::Counter);
        assert_eq!(ev.damage, HOOK.damage * 2.0);
        assert_eq!(d.counter_window, 0);
    }

    #[test]
    fn exhausted_fighter_staggers_and_cannot_block() {
        let (mut a, mut d) = pair();
        d.stamina = 2.0;
        assert!(!d.apply(Input::Block));
        assert!(!d.apply(Input::Dodge(Side::Left)));
        let ev = resolve(&mut a, &mut d, Action::Jab, true);
        assert_eq!(ev.kind, HitKind::Stagger);
        assert_eq!(d.state, State::Stunned(EXHAUSTED_STUN));
    }

    #[test]
    fn jab_interrupts_uppercut_startup() {
        let (mut a, mut d) = pair();
        d.state = State::Startup(Action::Uppercut, 10);
        let ev = resolve(&mut a, &mut d, Action::Jab, true);
        assert_eq!(ev.kind, HitKind::Interrupt);
        assert_eq!(d.state, State::Stunned(INTERRUPT_STUN));
    }

    #[test]
    fn stamina_regens_only_when_idle() {
        let mut f = Fighter::new("f");
        f.stamina = 50.0;
        f.state = State::Blocking(2);
        f.tick();
        assert_eq!(f.stamina, 50.0);
        f.state = State::Idle;
        for _ in 0..REGEN_DELAY {
            f.tick();
        }
        assert_eq!(f.stamina, 50.0, "no regen until the idle delay passes");
        f.tick();
        assert!(f.stamina > 50.0);
    }

    #[test]
    fn punch_timing_is_startup_then_recovery() {
        let mut f = Fighter::new("f");
        assert!(f.apply(Input::Jab));
        let mut landed = None;
        for i in 0..JAB.startup {
            landed = f.tick();
            if landed.is_some() {
                assert_eq!(i + 1, JAB.startup);
            }
        }
        assert_eq!(landed, Some(Action::Jab));
        assert_eq!(f.state, State::Recovery(Action::Jab, JAB.recovery));
    }

    #[test]
    fn match_is_deterministic_for_a_seed() {
        let run = |seed| {
            let mut m = Match::new(chainguard(), seed);
            for t in 0..(ROUND_TICKS * 3 + 2000) {
                if t % 23 == 0 {
                    m.player_input(Input::Jab);
                }
                if t % 97 == 0 {
                    m.player_input(Input::Dodge(Side::Left));
                }
                m.step();
            }
            (m.player.health, m.opp.health, m.phase)
        };
        assert_eq!(run(7), run(7));
        assert!(matches!(run(7).2, Phase::Over { .. }));
    }
}

#[cfg(test)]
mod balance {
    //! Headless balance probes. Run with `cargo test balance -- --nocapture`.
    use super::*;

    fn play(seed: u64, mut policy: impl FnMut(&Match) -> Option<Input>) -> (f32, f32, u8, bool, u32) {
        let mut m = Match::new(chainguard(), seed);
        let mut exhausted_ticks = 0;
        while !matches!(m.phase, Phase::Over { .. }) {
            if let Some(i) = policy(&m) {
                m.player_input(i);
            }
            m.step();
            if m.player.exhausted() {
                exhausted_ticks += 1;
            }
        }
        let won = matches!(m.phase, Phase::Over { player_won: true });
        (m.player.health, m.opp.health, m.round, won, exhausted_ticks / TICK_HZ)
    }

    #[test]
    fn report() {
        let masher = |_: &Match| Some(Input::Jab);
        let patient = |m: &Match| match m.opp.state {
            // Dodge any telegraphed punch, then counter with a hook.
            State::Startup(_, t) if t <= DODGE_WINDOW - 2 && m.player.can_act() => Some(Input::Dodge(Side::Left)),
            _ if m.player.counter_window > 0 && m.player.can_act() => Some(Input::Hook),
            // Guard break a blocker with an uppercut when safe.
            State::Blocking(t) if t > 4 && m.player.can_act() && m.player.stamina > 60.0 => Some(Input::Uppercut),
            State::Stunned(_) if m.player.can_act() => Some(Input::Hook),
            State::Idle if m.player.can_act() && m.player.stamina > 70.0 && m.tick % 7 == 0 => Some(Input::Jab),
            _ => None,
        };
        for (name, mut pol) in [("masher", Box::new(masher) as Box<dyn FnMut(&Match) -> Option<Input>>), ("patient", Box::new(patient))] {
            let mut wins = 0;
            let mut lines = Vec::new();
            for seed in 1..=10u64 {
                let (ph, oh, round, won, exh) = play(seed, &mut *pol);
                wins += won as u32;
                lines.push(format!("  seed {seed:>2}: {} in R{round}  you {ph:>5.1}  opp {oh:>5.1}  exhausted {exh}s", if won { "WIN " } else { "LOSS" }));
            }
            println!("{name}: {wins}/10 wins");
            for l in lines {
                println!("{l}");
            }
        }
    }
}

#[cfg(test)]
mod histogram {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn masher_events() {
        let mut m = Match::new(chainguard(), 1);
        let mut hist: BTreeMap<String, (u32, f32)> = BTreeMap::new();
        let mut opp_states: BTreeMap<String, u32> = BTreeMap::new();
        let mut landed_on: BTreeMap<String, u32> = BTreeMap::new();
        while !matches!(m.phase, Phase::Over { .. }) {
            let about_to_land = matches!(m.player.state, State::Startup(Action::Jab, 1));
            let opp_before = format!("{:?}", m.opp.state).split('(').next().unwrap_or("").to_string();
            m.player_input(Input::Jab);
            m.step();
            if about_to_land && m.events.iter().any(|e| e.by_player && e.kind == HitKind::Hit) {
                *landed_on.entry(opp_before).or_default() += 1;
            }
            if m.phase == Phase::Fighting {
                let k = match m.opp.state {
                    State::Idle => "Idle",
                    State::Startup(..) => "Startup",
                    State::Recovery(..) => "Recovery",
                    State::Blocking(_) => "Blocking",
                    State::Dodging(..) => "Dodging",
                    State::Stunned(_) => "Stunned",
                };
                *opp_states.entry(k.to_string()).or_default() += 1;
            }
            for ev in &m.events {
                let key = format!("{} {:?} {:?}", if ev.by_player { "P" } else { "O" }, ev.kind, ev.action);
                let e = hist.entry(key).or_default();
                e.0 += 1;
                e.1 += ev.damage;
            }
        }
        println!("ticks fought: {}", m.tick);
        for (k, (n, d)) in &hist {
            println!("  {k:<28} n={n:>3} dmg={d:>6.1}");
        }
        println!("opp state ticks: {opp_states:?}");
        println!("opp state when a player jab landed clean: {landed_on:?}");
    }
}
