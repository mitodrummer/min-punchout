//! The card, easiest to boss. Each opponent is one exaggerated trait, one
//! signature move, and a few lines. Public reputation as of mid 2026.

use crate::sim::{Input, Move, OppId, OpponentSpec, SpecialKind, Step, TICK_HZ};

fn ms(ms: u32) -> u32 {
    ((ms * TICK_HZ) as f32 / 1000.0).round().max(1.0) as u32
}

use Input::{Block, Hook, Jab, Uppercut};
use Step::{Do, Wait};

pub fn all() -> Vec<OpponentSpec> {
    vec![codespaces(), docker(), snyk(), bitnami(), devbox(), flox(), chainguard(), nix()]
}

pub fn by_name(name: &str) -> Option<OpponentSpec> {
    let n = name.to_ascii_lowercase();
    all().into_iter().find(|s| s.name.to_ascii_lowercase().starts_with(&n))
}


fn base(id: OppId, name: &'static str, nickname: &'static str, rank: u8) -> OpponentSpec {
    OpponentSpec {
        id,
        name,
        nickname,
        aliases: &[],
        kind: SpecialKind::BigPunch { stun_on_dodge: 45 },
        special_line: "WHIFF",
        prebuild: 0,
        fatigue: 1.0,
        health: 100.0,
        regen_per_sec: 20.0,
        speed: 1.2,
        tell_bonus: 6,
        block_chance: 0.4,
        dodge_chance: 0.1,
        reaction_delay: 3,
        patterns: vec![vec![Do(Jab), Wait(ms(600))]],
        special: Move { stamina: 30.0, damage: 28.0, startup: 40, recovery: 20 },
        special_every: 15 * TICK_HZ,
        special_name: "SPECIAL",
        special_short: "SPECIAL",
        taunts: ["", "", ""],
        ko_line: "",
        lose_line: "",
        rank,
        power: 1.0,
    }
}

/// 1. Decent fighter, but the meter never stops and he runs out of steam.
pub fn codespaces() -> OpponentSpec {
    OpponentSpec {
        health: 120.0,
        regen_per_sec: 18.0,
        speed: 1.6,
        tell_bonus: 12,
        block_chance: 0.3,
        dodge_chance: 0.1,
        reaction_delay: 5,
        power: 0.8,
        fatigue: 0.45,
        prebuild: 6 * TICK_HZ,
        patterns: vec![
            vec![Do(Jab), Wait(ms(900))],
            vec![Do(Hook), Wait(ms(1200))],
            vec![Do(Jab), Do(Jab), Wait(ms(1000))],
            vec![Wait(ms(800))],
        ],
        kind: SpecialKind::Freeze { ticks: 3 * TICK_HZ },
        special: Move { stamina: 0.0, damage: 0.0, startup: 1, recovery: 1 },
        special_every: 20 * TICK_HZ,
        special_name: "STOPPED DUE TO INACTIVITY",
        special_short: "IDLE",
        special_line: "RESUMING...",
        taunts: [
            "You've used 60 core hours this month.",
            "Billing starts the moment I stand up.",
            "Prebuild finished. Probably.",
        ],
        ko_line: "Your codespace has been stopped due to inactivity.",
        lose_line: "That round will appear on your next invoice.",
        ..base(OppId::Codespaces, "Codespaces", "The Minute Counter", 1)
    }
}

/// 2. Former champion. Huge, slow, and mostly a licensing department now.
pub fn docker() -> OpponentSpec {
    OpponentSpec {
        health: 150.0,
        regen_per_sec: 16.0,
        speed: 1.9,
        tell_bonus: 10,
        block_chance: 0.35,
        dodge_chance: 0.0,
        reaction_delay: 4,
        power: 1.0,
        patterns: vec![
            vec![Do(Hook), Wait(ms(1100))],
            vec![Do(Uppercut), Wait(ms(1400))],
            vec![Do(Jab), Wait(ms(700)), Do(Hook), Wait(ms(900))],
            vec![Do(Block), Wait(ms(600))],
        ],
        kind: SpecialKind::Burst { count: 6, cooldown: 6 * TICK_HZ },
        special: Move { stamina: 0.0, damage: 0.0, startup: 1, recovery: 1 },
        special_every: 14 * TICK_HZ,
        special_name: "RATE LIMIT",
        special_short: "RATE",
        special_line: "429: RETRY AFTER 6S",
        taunts: [
            "Please update Docker Desktop to continue.",
            "This round requires a Business subscription.",
            "Pulling... 4.2 GB... please wait.",
        ],
        ko_line: "This fight required a Business subscription.",
        lose_line: "Your free pulls have been exhausted.",
        ..base(OppId::Docker, "Docker", "The Desktop", 2)
    }
}

/// 3. Does not really hit you. Reports that you were hit, 1,400 times.
pub fn snyk() -> OpponentSpec {
    OpponentSpec {
        health: 90.0,
        regen_per_sec: 26.0,
        speed: 1.0,
        tell_bonus: 6,
        block_chance: 0.3,
        dodge_chance: 0.2,
        reaction_delay: 3,
        power: 0.4,
        patterns: vec![
            vec![Do(Jab), Do(Jab), Do(Jab), Wait(ms(300))],
            vec![Do(Jab), Wait(ms(200)), Do(Jab), Wait(ms(200))],
            vec![Do(Hook), Wait(ms(600))],
            vec![Do(Jab), Do(Jab), Do(Jab), Do(Jab), Wait(ms(500))],
        ],
        kind: SpecialKind::BigPunch { stun_on_dodge: 36 },
        special: Move { stamina: 20.0, damage: 60.0, startup: 14, recovery: 16 },
        special_every: 12 * TICK_HZ,
        special_name: "TRANSITIVE DEP",
        special_short: "DEP",
        special_line: "FALSE POSITIVE",
        taunts: [
            "Critical! Critical! Critical! (ok, that one's medium)",
            "1,400 issues found. Mostly in dev dependencies.",
            "Severity: high. Exploitability: none.",
        ],
        ko_line: "Severity of this loss: High. Fix available: no.",
        lose_line: "Your defeat has been added to the dashboard.",
        ..base(OppId::Snyk, "Snyk", "The Scanner", 3)
    }
}

/// 4. Fights fine until the free version is removed.
pub fn bitnami() -> OpponentSpec {
    OpponentSpec {
        health: 110.0,
        regen_per_sec: 20.0,
        speed: 1.4,
        tell_bonus: 6,
        block_chance: 0.5,
        dodge_chance: 0.1,
        reaction_delay: 3,
        power: 0.9,
        patterns: vec![
            vec![Do(Jab), Do(Hook), Wait(ms(700))],
            vec![Do(Block), Wait(ms(400))],
            vec![Do(Uppercut), Wait(ms(1000))],
            vec![Wait(ms(500))],
        ],
        kind: SpecialKind::Revoke { window: 5 * TICK_HZ },
        special: Move { stamina: 0.0, damage: 0.0, startup: 1, recovery: 1 },
        special_every: 12 * TICK_HZ,
        special_name: "DEPRECATION NOTICE",
        special_short: "REVOKE",
        special_line: "MOVED TO LEGACY",
        taunts: [
            "Your images will be removed on the 28th.",
            "This fighter is now part of the Broadcom portfolio.",
            "The free tier has been... adjusted.",
        ],
        ko_line: "This fighter has been moved to the legacy catalog.",
        lose_line: "Access to this ring now requires a subscription.",
        ..base(OppId::Bitnami, "Bitnami", "The Broadcom", 4)
    }
}

/// 5. Fast, scrappy, keeps changing his name mid-round.
pub fn devbox() -> OpponentSpec {
    OpponentSpec {
        aliases: &["Devbox", "Jetify", "Jetpack", "Devbox Cloud", "Jetify Devbox"],
        health: 80.0,
        regen_per_sec: 28.0,
        speed: 0.95,
        tell_bonus: 4,
        block_chance: 0.2,
        dodge_chance: 0.6,
        reaction_delay: 2,
        power: 0.8,
        patterns: vec![
            vec![Do(Jab), Do(Jab), Wait(ms(300))],
            vec![Do(Jab), Do(Hook), Wait(ms(400))],
            vec![Do(Hook), Wait(ms(350)), Do(Jab), Wait(ms(300))],
            vec![Wait(ms(250))],
        ],
        kind: SpecialKind::Rebrand { ticks: 45 },
        special: Move { stamina: 0.0, damage: 0.0, startup: 1, recovery: 1 },
        special_every: 10 * TICK_HZ,
        special_name: "REBRAND",
        special_short: "REBRAND",
        special_line: "SAME PRODUCT, NEW LOGO",
        taunts: [
            "We're basically Flox but we got there first!",
            "It's Nix, but you don't have to know that.",
            "Have you heard about our new name?",
        ],
        ko_line: "We'll be back under a new name.",
        lose_line: "Please update your bookmarks.",
        ..base(OppId::Devbox, "Devbox", "The Jetifier", 5)
    }
}

/// 6. Throws the exact same combo every time. So can you.
pub fn flox() -> OpponentSpec {
    OpponentSpec {
        health: 110.0,
        regen_per_sec: 22.0,
        speed: 1.25,
        tell_bonus: 6,
        block_chance: 0.6,
        dodge_chance: 0.2,
        reaction_delay: 2,
        power: 1.0,
        // One pattern, weight 1.0. Reproducible.
        patterns: vec![vec![
            Do(Jab),
            Do(Jab),
            Wait(ms(300)),
            Do(Hook),
            Wait(ms(500)),
            Do(Uppercut),
            Wait(ms(800)),
        ]],
        kind: SpecialKind::Freeze { ticks: 5 * TICK_HZ },
        special: Move { stamina: 0.0, damage: 0.0, startup: 1, recovery: 1 },
        special_every: 16 * TICK_HZ,
        special_name: "FLAKE CHECK",
        special_short: "FLAKE",
        special_line: "EVALUATED. SAME RESULT.",
        taunts: [
            "It's just Nix, but friendly!",
            "Same combo. Same result. That's the point.",
            "Evaluating... please hold.",
        ],
        ko_line: "That... should have worked on my machine.",
        lose_line: "Reproduced, as expected.",
        ..base(OppId::Flox, "Flox", "The Reproducer", 6)
    }
}

/// 7. Heavyweight. Blocks everything, chips at you, then sends a quote.
pub fn chainguard() -> OpponentSpec {
    OpponentSpec {
        health: 120.0,
        regen_per_sec: 24.0,
        speed: 1.5,
        tell_bonus: 8,
        block_chance: 0.9,
        dodge_chance: 0.05,
        reaction_delay: 2,
        patterns: vec![
            vec![Do(Hook), Wait(ms(500))],
            vec![Do(Jab), Do(Jab), Wait(ms(400))],
            vec![Do(Block), Wait(ms(300))],
            vec![Wait(ms(350))],
        ],
        kind: SpecialKind::BigPunch { stun_on_dodge: 60 },
        special: Move { stamina: 30.0, damage: 30.0, startup: 45, recovery: 20 },
        special_every: 15 * TICK_HZ,
        special_name: "ENTERPRISE QUOTE",
        special_short: "QUOTE",
        special_line: "QUOTE DECLINED",
        taunts: [
            "Zero CVEs. Zero free tier. Zero chance.",
            "Our guard is SLSA level 3.",
            "Have you considered a multi-year commitment?",
        ],
        ko_line: "Would you like to schedule a call to discuss your defeat?",
        lose_line: "We'll follow up with a revised quote.",
        ..base(OppId::Chainguard, "Chainguard", "The Zero Day", 7)
    }
}

/// 8. The boss. Unbelievably powerful, impossible to understand.
pub fn nix() -> OpponentSpec {
    OpponentSpec {
        health: 150.0,
        regen_per_sec: 30.0,
        speed: 0.9,
        tell_bonus: 2,
        block_chance: 0.4,
        dodge_chance: 0.4,
        reaction_delay: 1,
        power: 1.4,
        patterns: vec![
            vec![Do(Jab), Do(Uppercut), Wait(ms(300))],
            vec![Do(Hook), Do(Hook), Wait(ms(200))],
            vec![Do(Block), Do(Uppercut), Wait(ms(400))],
            vec![Wait(ms(150)), Do(Jab), Wait(ms(150)), Do(Jab), Wait(ms(150)), Do(Hook)],
            vec![Wait(ms(900))],
        ],
        kind: SpecialKind::BigPunch { stun_on_dodge: 30 },
        special: Move { stamina: 25.0, damage: 35.0, startup: 8, recovery: 14 },
        special_every: 9 * TICK_HZ,
        special_name: "DERIVATION",
        special_short: "DERIVE",
        special_line: "IMPURE",
        taunts: [
            "Have you considered writing it as a function?",
            "The documentation is wrong. The code is the documentation.",
            "This fight is reproducible. You lose every time.",
        ],
        ko_line: "Skill issue.",
        lose_line: "Garbage collected.",
        ..base(OppId::Nix, "Nix", "The Purist", 8)
    }
}
