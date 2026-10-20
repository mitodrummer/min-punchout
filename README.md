# punchout

Feel prototype for the Minimal Punch-Out terminal fighter. You are Minimal.
The only opponent so far is Chainguard, "The Zero Day": a heavyweight who
blocks nearly everything and, every fifteen seconds, winds up an Enterprise
Quote. Dodge it.

No art beyond boxes yet. The point of this build is to answer one question:
do the stamina numbers make patience win and mashing lose?

## Run

```sh
cargo run --release
cargo run --release -- --seed 7      # replayable opponent behaviour
cargo run --release -- --quiet       # no terminal bell
```

Needs a terminal at least 80x24.

| Key | Action |
|---|---|
| `j` | jab (fast, low damage, interrupts a heavy punch early in its wind-up) |
| `k` | hook |
| `l` | uppercut (slow, long tell, breaks a block) |
| `space` | block (tap, short window, chip damage from jabs and hooks) |
| `a` / `d` | dodge (tap, shorter window, opens a double-damage counter) |
| `tab` | debug line |
| `r` | rematch after the fight |
| `q` | quit |

## Rules in one breath

Every action costs stamina. Stamina only comes back after you have been idle
for a third of a second, and comes back at half speed once you have run dry.
Below the floor you cannot block or dodge and any hit staggers you. A dodge
followed by a punch inside the counter window does double damage.

## Tests

```sh
cargo test                                # rules + a headless 80x24 render
cargo test balance -- --nocapture         # masher vs patient, ten seeds each
cargo test histogram -- --nocapture       # event counts for a masher fight
```

The balance probes are the tuning loop. Change a number in `src/sim.rs`,
rerun, read the table.

## Layout

- `src/sim.rs`: the whole game, no terminal dependency, 30 ticks per second
- `src/render.rs`: sim state to ratatui widgets
- `src/main.rs`: input, tick loop, bell

## Building in a Minimal sandbox

`min add --session rust gcc binutils`, then `export CARGO_HOME=/state/cargo`.
There is no `cc` alias in the sandbox, so `.cargo/config.toml` names `gcc`
as the linker.
