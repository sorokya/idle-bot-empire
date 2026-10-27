# Idle Bot Empire

A text-first idle game about running a gold-farming bot empire inside a fictional MMORPG.
Design doc: `~/vault/20. Goals & Projects/Projects/Idle Bot Empire.md`

## Layout

```
crates/sim/     # the game. No I/O, no clock, no unseeded randomness.
  balance.toml  # every tunable number
crates/wasm/    # wasm-bindgen boundary (~60 lines)
web/            # Vite + TypeScript. Dumb view layer.
```

## Run it

```bash
# balance harness: scripted player, prints a full run log
cargo run --example run -- 6          # 6 sim hours, default seed

# tests (determinism, save/load, offline cap, pacing bands)
cargo test -p ibe-sim

# wasm build
cd crates/wasm && wasm-pack build --target web --release --out-dir ../../web/src/wasm

# ui
cd ../../web && npm install && npm run dev
```

## Architecture rules

1. `sim` never reads a clock. The host passes `elapsed_ms` in. Offline catch-up and
   live play run the same code path.
2. The RNG is PCG32 with its state inside the save, so a reload replays identically.
3. The UI reads a flat `ViewModel`; it never touches `State`.
4. Every number lives in `balance.toml`, not in code.
5. Time advances in fixed 1s ticks, accumulated — a 12h catch-up is 43,200 cheap iterations.

## Controls

`1`–`6` buy · `[` `]` aggression · `p` new identity · click an account to pause or burn.

## Where the design doc and the code disagree

The slice deliberately ships less than the doc: one zone, one shard, no manual
clicking phase, no laundry, no buyers, no rivals. It does ship the part that
matters — heat, sweeps, and Contacts.
