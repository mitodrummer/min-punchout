# minimal punchout

Feel prototype for the Minimal Punch-Out terminal fighter. You are Minimal,
seen from behind at the bottom of the ring. Each opponent on the card fails one
of the four things Minimal is for: isolation, reproducibility, supply chain,
hermetic builds. Beat them and the card keeps score of what you proved. The
card, easiest first:

| # | Opponent | Trait | Signature move |
|---|---|---|---|
| 1 | Codespaces, "The Rebuild" | reproducibility: a devcontainer that is different every rebuild | Stopped Due To Inactivity: stands still for 3 s. Proves: PINNED |
| 2 | Docker, "The Latest Tag" | isolation and hermetic builds: shares your kernel, FROM ubuntu:latest | Rate Limit: six quick jabs, then nothing for 6 s. Proves: HERMETIC |
| 3 | Snyk, "The Afterthought" | supply chain: finds it after you shipped it | Transitive Dep: a punch from a glove you never saw. Proves: ATTESTED |
| 4 | Bitnami, "The Provenance" | supply chain: built somewhere, by someone | Deprecation Notice: posts a notice, then heals back the last 5 s of your damage. Hit him while it is posted to cancel it. Proves: SOURCE PINNED |
| 5 | Devbox, "The Houseguest" | isolation: runs on your host, next to your keys | Rebrand: slips away and comes back under a new name. Proves: ISOLATED |
| 6 | Flox, "The Asterisk" | reproducible, on top of whatever your laptop is | Flake Check: evaluates for 5 s, free hits. Proves: SAME VM. EVERY TIME. |
| 7 | Chainguard, "The Base Image" | supply chain, image only: zero CVEs in the base, your build is on its own | Enterprise Quote: a briefcase to the face, dodge it. Proves: WHOLE CHAIN |
| 8 | Nix, "The Purist" | reproducible if you pin; nobody pins; every few seconds his own build fails and he punches himself in the face | Derivation: you will not see it coming. Proves: IMPURE |

The game opens on a title screen, then the card: all eight in ladder order, the next one up
flashing, beaten ones stamped KO, the rest in silhouette. Win and the card
comes back with the next slot lit. The point of this build is to answer one
question: do the stamina numbers make patience win and mashing lose?

## Run

```sh
cargo run --release
cargo run --release -- --opponent docker   # unlock the card up to a named opponent (a prefix is enough)
cargo run --release -- --log session.log   # record every key press and hit for tuning
cargo run --release -- --no-music          # effects only
cargo run --release -- --rank 3            # dev dial: re-tune the opponent to ladder rank 1 (easy) to 8 (boss)
cargo run --release -- --seed 7            # replayable opponent behaviour
cargo run --release -- --quiet             # no sound at all
```

Needs a terminal of at least 100x40 with true colour (any modern terminal). The ring
scales up in integer steps as the window grows, so a 200x80 window gets 2x pixels.

| Key | Action |
|---|---|
| `enter` | title screen: start. On the card: fight the lit opponent. After a fight: back to the card |
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
full, and the last stretch of the fuse track is drawn dark red: that is the
press zone. While the fill is inside it the word DODGE (or BLOCK, for a jab)
flashes over his chest. Press at any point while it shows; you do not need
to wait for the fuse to fill. Jabs are quick enough that they flash at once. Hooks
pull the glove far out to one side, uppercuts crouch with both gloves low,
jabs cock one glove up by the ear. The punch name sits in both bottom
corners, and with hints on the footer names the right answer.

## Rules in one breath

A press made while your last move is still finishing is held for a quarter
second and fires the moment you are free, so mashing does not eat inputs.
Rounds are 35 seconds, three of them, with a breather between.
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

`min add --session rust gcc binutils alsa-lib pkgconf`, then `export CARGO_HOME=/state/cargo`.
There is no `cc` alias in the sandbox, so `.cargo/config.toml` names `gcc`
as the linker.

## Sound

Chiptune, synthesized at startup from two pulse channels, a triangle bass
and a noise drum, the NES recipe. There is a card theme, a fight loop, and
an effect for every hit, block, dodge, stun, the bell, and the press cue:
a double blip the instant the opponent turns red, so you can play it by
ear. Nothing on disk. It plays through the OS mixer via `rodio`, so run it
on a machine with speakers (a Mac host is fine); where no audio device
opens, the game falls back to the terminal bell. `--no-music` keeps the
effects, `--quiet` silences everything.

Building on Linux needs the ALSA headers (`alsa-lib` and `pkgconf` in a
Minimal sandbox). macOS and Windows need nothing extra.
