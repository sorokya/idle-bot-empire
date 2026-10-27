# AGENTS.md

Text-first idle game: pure Rust sim -> wasm-bindgen -> Preact/TypeScript UI.
`docs/design.md` records design intent; code, `balance.toml`, and tests are the
executable source of truth. Run everything from the repo root unless noted.

## Commands

```bash
# auto-fix / check
cargo fmt --all
cargo fmt --all -- --check
cd web && pnpm format                              # biome check --write
cd web && pnpm lint                                # biome check

# verify
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p ibe-sim                              # sim tests only
cargo test -p ibe-sim --test pacing                # balance gate only
cargo test -p ibe-sim --test sim_tests same_seed_same_result   # single test
cd web && pnpm typecheck                           # tsc --noEmit
cd web && pnpm build                               # typecheck + vite build
cargo run --example run -- 6 7                     # balance log: hours, seed

# wasm: required after sim/ViewModel/balance changes before UI work
cd crates/wasm && wasm-pack build --target web --release --out-dir ../../web/src/wasm

cd web && pnpm install && pnpm dev                 # http://localhost:3000
```

## Quality gates

- Rules are mechanically enforced where possible. A format, lint, type, or test
  failure blocks completion; never weaken config or add a suppression to pass.
- Auto-fix is allowed without asking: run `cargo fmt --all` and, for web files,
  `pnpm format`, then re-run the checks and inspect the diff. Fix only the
  touched area; do not reformat unrelated files.
- Rust changes must pass `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace`.
- Web changes must pass `pnpm lint`, `pnpm typecheck`, and `pnpm build`.
- If a change touches `ViewModel`, `Command`, the wasm boundary, or generated
  wasm, run both stacks and rebuild wasm. CI still runs the full Rust and Web
  jobs on every push/PR.
- Pre-commit runs format + Biome; pre-push runs clippy + `cargo test`.
- `rust-analyzer` is installed via the pinned toolchain and used for diagnostics,
  navigation, and refactors. `cargo check`, clippy, and CI remain the source of
  truth; editor diagnostics do not replace the CLI gates.
- Suppressions (`#[allow]`, `biome-ignore`) must use the narrowest scope and
  include a reason. No crate-wide or file-wide blanket allows.
- These rules are repo-wide. Keep mass renames and structural cleanup
  behavior-preserving, and do not mix them with feature work.

## Code standards

### General

- Comments explain why, invariants, and tradeoffs. No narration or
  commented-out code. Public Rust items get doc comments.
- Functions: soft 60 lines, hard 100. Extract helpers before 100; one
  responsibility per function.
- Files: soft 400 lines, hard 800. Split by domain past 800.
- Naming: fully descriptive. `balance`, `aggression`, `state`, `view`, not
  `bal`, `aggr`, `s`, `v`. Standard acronyms (`id`, `rng`, `wasm`, `json`,
  `toml`) are allowed.
- Errors: fallible engine operations return a typed error enum, not `String`.
  No `unwrap`/`expect` in library paths except proven invariants with a message.
  Tests may unwrap. Never panic for a player-facing failure.
- No domain newtypes: keep descriptive primitives and document units. Do not
  introduce `Gold`/`Cash`/`AccountId` wrappers; this was an explicit choice.
- Tests: every behavior change needs a test; every bug fix gets a regression
  test that failed before the fix. Sim tests are deterministic; no clock,
  network, sleeps, or unseeded randomness. Integration tests live in
  `crates/sim/tests`; unit tests sit next to pure helpers.
- Dependencies: ask before adding a crate or npm package; prefer std and
  existing packages. No `unsafe` in repo code.
- Docs: update `AGENTS.md` and `README.md` when commands or architecture change;
  update `docs/design.md` when a design decision or shipped/planned status
  changes.

### Rust

- Player actions are a typed `Command` enum; parse strings only once at the wasm
  boundary. Do not add new string-only actions to the engine.
- Numeric `as` is allowed only in proven-range sim math. Use `From`/`TryFrom`
  for IDs, lengths, and boundary conversions; add helpers for time/level
  conversions instead of open-coded casts.
- Default to private; `pub(crate)` unless wasm or tests need it. Every `pub`
  item and module gets docs.
- Keep the sim pure: no clock, I/O, or ambient randomness. `Game::tick` is the
  only time path; PCG32 state lives in `State`.

### TypeScript

- No `any`, no non-null `!`, and no casts except at the wasm boundary
  (`game.view() as ViewModel`) and `import.meta.env`. Use `unknown` + narrowing.
- `tsconfig` adds `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`.
- `main.tsx` only mounts. Put ViewModel types in `types.ts`, state/actions in
  `useGame.ts`, formatting helpers in `format.ts`, and one component per file
  under `components/`.
- camelCase for values/functions, PascalCase for components/types,
  UPPER_SNAKE_CASE for module constants. Prefer `interface` for object shapes
  and `type` for unions/aliases.
- The UI is a dumb view: no game rules in TypeScript; behavior comes from the
  wasm `ViewModel`/`Command` API.
- There is no UI test runner yet. Do not add Vitest/Playwright until real UI
  work begins; verify with `pnpm build`, Biome, a wasm rebuild, and a manual
  smoke check.

## Architecture invariants

- `crates/sim` has no clock, I/O, or ambient randomness. The host passes elapsed
  time; `Game::tick(delta_seconds)` is the only way time advances, for offline
  catch-up and live play alike. RNG is PCG32 with its state in `State`, so a save
  replays identically.
- `tick` uses fixed <=1s steps and clamps every call to `[offline].cap_secs`
  (12h). A single huge tick stops at 12h; it does not run longer.
- Economy tunables live in `crates/sim/balance.toml`, embedded at compile time
  with `include_str!`; keep `Balance` in `balance.rs` in sync. A few behavioral
  constants (account age factor, burn salvage) remain hardcoded in `engine.rs`.
- `State` is serde JSON. Bump `SAVE_VERSION` in `crates/sim/src/lib.rs` when its
  shape changes; additive fields should use `#[serde(default)]`. Browser save key
  is `idle-bot-empire/save/v1`.
- The UI only sees `ViewModel` from `view.rs`, serialized with
  `serde_wasm_bindgen` (snake_case). The `ViewModel` mirror in
  `web/src/types.ts` is hand-written: update it whenever `ViewModel` changes.
  There is no codegen.
- `engine.rs` is a module root; tick, actions, sweep, save, and support helpers
  live under `crates/sim/src/engine/`.
- Action vocabulary: `buy_account`, `buy_slot`, `sell`, `click`,
  `upgrade:script|proxy|humanizer`, `pause:<id>`, `burn:<id>`, `aggression:<n>`,
  `face_sweep`, `prestige`, `wait:<secs>`. These are `Command` variants; strings
  are parsed only at the wasm boundary.
- Manual phase lasts until the first bot: only `click`, `sell`, `buy_account`
  are meaningful then; do not add automation shortcuts to it.

## Design doc and scope

- `docs/design.md` is the design reference. Anything marked *(planned)* is not
  built: multiple shards, buyer orders, rivals, laundering, aged-account broker,
  milestone multipliers, Contact tree, `why` panel, saved loadouts, narrative
  events, graphics, sound, achievements. Do not implement them unless asked.
- Preserve the fairness rules when touching the engine or UI: a sweep queues and
  waits for `face_sweep` (offline catch-up must never make that decision for the
  player), the next sweep is always visible, and an empty account list always
  respawns the personal account.
- Log/UI copy is deadpan bureaucratic satire; keep the MMO fictional and never
  reference real games or real-money trading.

## Generated and ignored files

- `web/src/wasm/` is generated by wasm-pack and gitignored. A fresh checkout
  fails `pnpm dev`/`pnpm build` until the wasm build above is run.
- Ignored: `target/`, `node_modules/`, `web/dist/`, `web/src/wasm/`.

## Balance and test quirks

- `crates/sim/tests/pacing.rs` is a balance gate, not a correctness test: seed 7,
  first run must end in 2-5h, first prestige pays 2-40 contacts, run two is not
  >2x run one, and early play never goes >45 min without an affordable purchase.
  Re-run it after any `balance.toml`, tick, RNG, or economy change; do not widen
  the bands just to make a change pass.
- `sim_tests.rs` compares saved JSON from identical seeded runs; any hidden
  clock/randomness breaks it.
- The `ibe-wasm` crate has parser tests; verify the full boundary by building
  wasm and loading the UI; in Vite dev mode `window.ibe` exposes `ff(secs)`,
  `view()`, and `reset()` for skipping time and clearing the save.

## Tooling notes

- Rust is pinned to 1.99.1 with rustfmt, clippy, rust-analyzer, and the
  `wasm32-unknown-unknown` target (`rust-toolchain.toml`); `rustup show` installs
  it. `rustfmt.toml`, `.editorconfig`, `Cargo.toml` lint tables, `clippy.toml`,
  and stricter `biome.json` are the mechanical enforcement points.
- pnpm is pinned to 12.15.0 via `packageManager`; CI uses Node 24 and
  `pnpm install --frozen-lockfile`, so commit `web/pnpm-lock.yaml` with
  dependency changes. Biome 2.6.3 is the JS linter/formatter (single quotes,
  organized imports) and additionally enforces the TypeScript safety rules.
- lefthook runs fmt + web lint on commit and clippy + `cargo test` on push; CI
  also runs `pnpm typecheck`, `pnpm build`, and the wasm build.
