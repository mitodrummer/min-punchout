# punchout

Feel prototype for the Minimal Punch-Out terminal fighter. You are Minimal,
seen from behind at the bottom of the ring. The card, easiest first:

| # | Opponent | Trait | Signature move |
|---|---|---|---|
| 1 | Codespaces, "The Minute Counter" | spends the opening prebuilding, fades as the core hours burn | Stopped Due To Inactivity: stands still for 3 s |
| 2 | Docker, "The Desktop" | huge, slow, hits hard | Rate Limit: six quick jabs, then nothing for 6 s |
| 3 | Snyk, "The Scanner" | tiny jabs, hundreds of them, every one labelled CRITICAL | Transitive Dep: a punch from a glove you never saw |
| 4 | Bitnami, "The Broadcom" | ordinary fighter with an unfair contract | Deprecation Notice: posts a notice, then heals back the last 5 s of your damage. Hit him while it is posted to cancel it |
| 5 | Devbox, "The Jetifier" | thin, fast, dodges everything | Rebrand: slips away and comes back under a new name |
| 6 | Flox, "The Reproducer" | the exact same combo every time | Flake Check: evaluates for 5 s, free hits |
| 7 | Chainguard, "The Zero Day" | blocks 90% of everything | Enterprise Quote: a briefcase to the face, dodge it |
| 8 | Nix, "The Purist" | boss; hits like a truck with almost no tell | Derivation: you will not see it coming |

The game opens on the card: all eight in ladder order, the next one up
flashing, beaten ones stamped KO, the rest in silhouette. Win and the card
comes back with the next slot lit. The point of this build is to answer one
question: do the stamina numbers make patience win and mashing lose?

## Run

```sh
cargo run --release
cargo run --release -- --opponent docker   # unlock the card up to a named opponent (a prefix is enough)
cargo run --release -- --log session.log   # record every key press and hit for tuning
cargo run --release -- --rank 3            # dev dial: re-tune the opponent to ladder rank 1 (easy) to 8 (boss)
cargo run --release -- --seed 7            # replayable opponent behaviour
cargo run --release -- --quiet             # no terminal bell
```

Needs a terminal of at least 100x40 with true colour (any modern terminal). The ring
scales up in integer steps as the window grows, so a 200x80 window gets 2x pixels.

| Key | Action |
|---|---|
| `enter` | on the card: fight the lit opponent. After a fight: back to the card |
| `a` / `d` | on the card: pick among the unlocked opponents (rematches allowed) |
| `j` | jab (fast, low damage, interrupts a heavy punch early in its wind-up) |
| `k` | hook |
| `l` | uppercut (slow, long tell, breaks a block) |
| `space` | block (tap, short window, chip damage from jabs and hooks) |
| `a` / `d` | dodge (tap, shorter window, opens a double-damage counter) |
| `h` | toggle the trainer hints in the footer |
| `tab` | debug line and event log |
| `r` | rematch, after the fight |
| `q` | quit |

## Reading the opponent

Every punch has a wind-up pose. While the punch is still far off the
opponent turns yellow and a fuse bar fills under the sponsor banner: wait.
He flashes red exactly when a dodge or block pressed now will still be up
when the punch lands, so red means press. The punch lands when the fuse is
full. Jabs are quick enough that they flash red at once. Hooks
pull the glove far out to one side, uppercuts crouch with both gloves low,
jabs cock one glove up by the ear. The punch name sits in both bottom
corners, and with hints on the footer names the right answer.

## Rules in one breath

A press made while your last move is still finishing is held for a quarter
second and fires the moment you are free, so mashing does not eat inputs.
Every action costs stamina. Stamina only comes back after you have been idle
for a third of a second, and comes back at half speed once you have run dry.
Below the floor you cannot block or dodge and any hit staggers you. A dodge
followed by a punch inside the counter window does double damage.

## Tests

```sh
cargo test                                # rules + headless renders
cargo test renders -- --nocapture         # print half-block frames as text
PUNCHOUT_FRAMES=/tmp/frames cargo test dump_frames   # one PNG per pose and per opponent
cargo test balance -- --nocapture         # masher vs patient, ten seeds each
cargo test histogram -- --nocapture       # event counts for a masher fight
cargo test ranks -- --nocapture           # a sloppy human vs Chainguard at every rank
cargo test card -- --nocapture            # a sloppy human vs every opponent at its own rank
```

The balance probes are the tuning loop. Change a number in `src/roster.rs`
or `src/sim.rs`, rerun, read the table.

## Layout

- `src/sim.rs`: the whole game, no terminal dependency, 30 ticks per second
- `src/roster.rs`: the eight opponents: stats, patterns, specials, lines
- `src/pixel.rs`: pixel canvas, half-block flush, 3x5 font, test-only PNG writer
- `src/art.rs`: fighter rigs, looks and poses (a pose is coordinates, not a drawing)
- `src/render.rs`: the Punch-Out layout, bars, banner, overlays
- `src/main.rs`: input, tick loop, bell, opponent selection

## Building in a Minimal sandbox

`min add --session rust gcc binutils`, then `export CARGO_HOME=/state/cargo`.
There is no `cc` alias in the sandbox, so `.cargo/config.toml` names `gcc`
as the linker.
