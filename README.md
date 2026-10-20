# punchout

Feel prototype for the Minimal Punch-Out terminal fighter. You are Minimal,
seen from behind at the bottom of the ring. The only opponent so far is
Chainguard, "The Zero Day": a heavyweight who blocks nearly everything and,
every fifteen seconds, winds up an Enterprise Quote. Dodge it.

The point of this build is to answer one question: do the stamina numbers
make patience win and mashing lose?

## Run

```sh
cargo run --release
cargo run --release -- --seed 7      # replayable opponent behaviour
cargo run --release -- --quiet       # no terminal bell
cargo run --release -- --rank 3      # dev dial: re-tune the opponent to ladder rank 1 (easy) to 8 (boss)
```

Needs a terminal of at least 100x40 with true colour (any modern terminal). The ring
scales up in integer steps as the window grows, so a 200x80 window gets 2x pixels.

| Key | Action |
|---|---|
| `j` | jab (fast, low damage, interrupts a heavy punch early in its wind-up) |
| `k` | hook |
| `l` | uppercut (slow, long tell, breaks a block) |
| `space` | block (tap, short window, chip damage from jabs and hooks) |
| `a` / `d` | dodge (tap, shorter window, opens a double-damage counter) |
| `h` | toggle the trainer hints under the ropes |
| `tab` | debug line and event log |
| `r` | rematch after the fight |
| `q` | quit |

## Reading the opponent

Every punch has a wind-up pose. During the first half of the wind-up the
opponent turns yellow and a fuse bar fills under the sponsor banner. During the second
half he flashes red: the punch lands when the fuse is full. Hooks pull the
glove far out to one side, uppercuts crouch with both gloves low, jabs cock
one glove up by the ear, and the Enterprise Quote holds up a gold briefcase
while the price ticks up. With hints on, a line under the ropes names the
punch and the right answer in the footer.

## Rules in one breath

Every action costs stamina. Stamina only comes back after you have been idle
for a third of a second, and comes back at half speed once you have run dry.
Below the floor you cannot block or dodge and any hit staggers you. A dodge
followed by a punch inside the counter window does double damage.

## Tests

```sh
cargo test                                # rules + headless 80x24 renders
cargo test renders -- --nocapture         # print half-block frames as text
PUNCHOUT_FRAMES=/tmp/frames cargo test dump_frames   # write one PNG per pose
cargo test balance -- --nocapture         # masher vs patient, ten seeds each
cargo test histogram -- --nocapture       # event counts for a masher fight
cargo test ranks -- --nocapture           # a sloppy human vs the opponent at every rank
```

The balance probes are the tuning loop. Change a number in `src/sim.rs`,
rerun, read the table.

## Layout

- `src/sim.rs`: the whole game, no terminal dependency, 30 ticks per second
- `src/pixel.rs`: pixel canvas, half-block flush, 3x5 font, test-only PNG writer
- `src/art.rs`: fighter rigs, palettes and poses (a pose is coordinates, not a drawing)
- `src/render.rs`: the Punch-Out layout, bars, banner, overlays
- `src/main.rs`: input, tick loop, bell

## Building in a Minimal sandbox

`min add --session rust gcc binutils`, then `export CARGO_HOME=/state/cargo`.
There is no `cc` alias in the sandbox, so `.cargo/config.toml` names `gcc`
as the linker.
