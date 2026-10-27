# Idle Bot Empire — Design

> **Pitch:** You don't play the MMORPG. You own the bots that play it.
> Build a gold-farming operation, sell the gold on the black market, upgrade the infrastructure, and survive the ban wave. Then do it again, faster.

**Status:** playable vertical slice. Design sections marked *(planned)* are not built yet.
**Code:** [sorokya/idle-bot-empire](https://github.com/sorokya/idle-bot-empire) — see `README.md` to run it.
**Balance:** economy tunables live in [`crates/sim/balance.toml`](../crates/sim/balance.toml); a few behavioral constants remain in engine code.

---

## 0. Where it stands

Shipped and verified:

- Manual phase → first bot → accounts → gold → sell → cash → slots / scripts / proxies / humanizer
- Heat tiers with warnings, pause/burn recovery, instant termination at 100 heat
- Queued sweep at the exposure ceiling → Contacts → new identity with a permanent yield multiplier
- 12h offline accrual, deterministic saves, 14 Rust tests, Preact UI

Built but thin: one shard, one zone, no buyers, no rivals, no laundry.

---

## 1. The Thesis

An idle game about automation is the genre eating itself — that's the joke, and it's also the design.

The player's job is to **optimize a system that runs without them**. That is literally what an idle game asks of a player, so the fiction and the mechanic are the same thing. The theme isn't a skin over "numbers go up"; it's the reason the numbers go up.

Three consequences fall out of this, and they're the pillars:

**Pillar 1 — The player is the bottleneck, not the clock.**
The game should feel like *"my operation is running, and I'm deciding what to do with the output."* Never *"I am clicking to make progress."* Clicking is a tutorial phase, not the game — and the build now enforces that: hand-farming lasts about nineteen clicks and then never matters again.

**Pillar 2 — Risk is the strategic dial.**
Almost every idle game's only decision is "buy the thing that increases the number." Ours has two dials pulling opposite ways: **throughput vs. heat**. Every choice — more accounts, more hours, faster selling — buys gold with exposure. That's a real decision every session, and it's the theme.

**Pillar 3 — The ban is the prestige.**
Bans are the genre's reset mechanic with a narrative reason to exist. You lose the accounts (run state) and keep the knowledge, contacts, and capital (meta state). Like *Trailer Park Boys: Greasy Money*, where ascension is "the boys got thrown in jail again" — the reset is the story, not a menu.

---

## 2. What the Research Says

Synthesized from r/incremental_games threads (2017–2026), GDC talks on idle math, and design write-ups on Cookie Clicker / AdVenture Capitalist.

### What players love

- **"Numbers go up, but in a way I can influence with smart decisions."** The single most-upvoted answer in every thread. Raw growth is table stakes; *agency over* growth is the product.
- **Discovery / unfolding.** Start with one verb. Reveal mechanics over hours. Players repeatedly cite *Universal Paperclips* and *Theory of Magic*: "the mystery of what comes next" is the hook. The anti-pattern is dumping a 12-panel UI on minute one.
- **Increasing automation as a reward.** "Good loops, several prestige layers, and increasing unlocks of automation." Automating a thing you used to do manually is the best feeling in the genre.
- **Steady progress, soft walls.** Slow is fine. *Stalled* is not. "If it takes too long to progress when you get stuck, it sucks out all the fun."
- **The return.** The emotional beat is the "while you were away" payout. It should be a designed moment, not a receipt.
- **One big legible number**, clean responsive controls, honest upgrade text.

### What players hate

- **Offline caps.** Top-voted complaint, repeatedly. An arbitrary 2-hour cap turns an idle game into a chore timer.
- **Unrecoverable mistakes.** "A way for a player to mess up so badly they're better off hard-resetting." Instant churn.
- **Prestige loops.** Once runs get short enough that you must reset every 10 minutes, "it's not really an idle game anymore."
- **Re-doing setup after every reset.** Micromanagement that gets wiped each prestige is the #2 prestige complaint.
- **Obscure upgrade text.** "'Infectivity' boosts 'Deadly' — currently ^e6.650e15" is the canonical bad example. If the player needs a wiki, the design failed.
- **Prestige currency that's just a speed-up.** "If it takes 6 hours and a reset to get a 50% bonus, I'm uninstalling."
- **Numbers rising faster than the player's contribution.** "I want the numbers to go up *because I made them go up*." Returning to a pile of unearned upgrade currency feels like homework.
- **Prestige-for-its-own-sake**, and prestige points that give passive bonuses for being *unspent* (punishes buying upgrades).
- **Ads, energy, mandatory IAP.**

### The math that makes it work

- Costs grow geometrically (×1.07–1.15 per purchase). Income grows by *multiplying factors* (units × milestones × upgrades × prestige).
- **Tune the interval, not the magnitude.** Target ~1–2 minutes to the next meaningful purchase, at every scale. Players feel the wait, not the exponent.
- Milestone multipliers at 25/50/100 owned keep the "buy more" button exciting. *(planned — not in the slice)*
- Prestige conversion uses a **fractional exponent** (cube root for Cookie Clicker, sqrt for AdVenture Capitalist), so doubling prestige currency requires 4–8× the run.
- Common reset heuristic: prestige when the gain is **+50% to +200%**.

### Cadence to design for

Players describe two healthy modes: **1–3 minute check-ins every hour or two** (Kittens Game) and **15–20 minute active sessions** (Idle Wasteland). Target: *short check-ins that always have one good decision in them*, with a longer session when something unlocks.

---

## 3. Core Loop

```
        ┌──────────────────────────────────────────────┐
        │  0. FARM BY HAND  →  the first 19 clicks     │
        │  1. BOTS FARM     →  gold/hr (dirty)         │
        │  2. SELL GOLD     →  cash (market price)     │
        │  3. BUY           →  accounts, slots,        │
        │                      scripts, countermeasures│
        │  4. HEAT RISES with every account-hour       │
        │  5. MANAGE        →  pause, rotate, burn     │
        │  6. SWEEP         →  lose the run, keep the  │
        │                      meta → back to 1, faster│
        └──────────────────────────────────────────────┘
```

The loop is a **thermostat, not a treadmill**. You're not just buying more; you're buying more *and* paying for the safety that lets you keep it.

### The cycle by timescale

- **30 seconds:** a bot levels up, a batch finishes, a price ticks.
- **5 minutes:** a sale decision, a flagged account, an upgrade.
- **1–2 hours:** a ban warning you have to act on; a shard price recovering.
- **1–3 days:** a run approaching its ceiling; a prestige decision. *(later tiers)*
- **1–2 weeks:** a meta-layer unlock — second shard, cartel, laundering front. *(planned)*

---

## 4. Resources

Keep it to **four numbers plus one meter** in the MVP. Every extra currency is a UI tax and a balance liability.

| Resource | Source | Sink | Feel |
|---|---|---|---|
| **Gold (g)** | Bots farming a shard | Sold on market | The treadmill. Big, dirty, visible. |
| **Cash ($)** | Selling gold | Everything | The real number. The one that survives a ban. |
| **Accounts** | Bought with cash, levelled over time | Banned, burned | Consumable units. The "generators." |
| **Slots** | Rack space, bought with cash | Cap on accounts | The tier gate. Prevents runaway width. |
| **Heat** | Everything | Never spent — only cooled | The pressure. 0–100 per account, plus operation-wide Exposure. |

**Contacts** are the prestige currency and are shipped. **Reputation** (buyer access) is the next layer *(planned)*.

### Why accounts, not "bots"

Accounts are the interesting unit: they have **age** (older = trusted = lower heat per hour), **level** (higher = more gold/hr), and **identity**. Buying an aged account would be expensive and quiet; levelling a fresh one is cheap and loud. Age-based stealth is implemented; the aged-account broker is *(planned)*.

---

## 5. The Core Tension: Throughput vs. Heat

This is the game. Everything else is scaffolding.

**Gold/hr** is a function of: accounts × level × script × contacts multiplier × aggression.
**Heat/hr per account** is a function of: crowding × aggression ÷ (proxy × humanizer × account age).

The player sets the aggression. Low and slow is safe and boring; max throughput is a countdown.

### Heat model (per account, as shipped)

```
heat_gain/hr = 12 × (1 + 0.06 × active_accounts) × aggression^1.7 / stealth
stealth      = proxy × humanizer × age_factor
age_factor   = 1 + (age_days / 120), capped at 2.5
proxy        = 0.70 + 0.28 × proxy_level
humanizer    = 0.80 + 0.22 × humanizer_level
```

- `aggression` 0.2–2.0 — player-set, global. Multiplies gold/hr linearly but heat **super-linearly**: greed is strictly inefficient per gold, and only worth it when you want throughput before a sweep.
- Proxies and humanizer are the two stealth knobs; both are pure cash sinks that raise `stealth`.
- Age rewards keeping an account alive — an old account is cheap heat.

### Heat states (readable, with warnings)

Gradual meter, clear tiers, decay with a delay, never a surprise.

| Heat | State | What the player sees |
|---|---|---|
| 0–39 | **Quiet** | Nothing. |
| 40–69 | **Flagged** | `GM opened a ticket on #A17 (heat 41). Nothing happened. Yet.` Bar turns amber. |
| 70–89 | **Watched** | `#A17 is being watched (heat 72). Est. action within a few hours.` Bar turns red. |
| 90–99 | **Imminent** | `#A17 will be actioned shortly (heat 93). Move it or lose it.` |
| 100 | **Terminated** | Account removed on the spot, mid-session. No further warning. |

Rules that make this fair:

1. **The warning is the window.** At 90 the account is not gone — it is *countdown*. At your current heat rate, the bar tells you how long you have. Only ignoring it to 100 kills the account outright.
2. **Decay needs a delay.** Pausing doesn't cool instantly — 10 minutes of "the GM is still looking" first, then 30 heat/hr. Prevents meter-flicker exploits and makes rotation viable.
3. **Recovery always exists.** Pause (heat decays) or **burn** — delete the account and salvage 40% of its value as clean cash. Burning is the pressure valve that guarantees no dead ends. Shard migration is *(planned)*.
4. **No account, no dead end.** If every account dies, you log back in on your personal one and start again. *(planned: the `why` itemisation)*

### Operation-wide Exposure

Individual account heat is tactical. **Exposure** is strategic: it accrues with bot-hours across the whole operation, **never decays**, and drives the sweep.

```
exposure/hr = 5.5 × active_accounts × aggression^1.2
ceiling     = 100 × 1.3^tier
```

Growth is self-limiting, and the limit is the story. Pausing accounts slows exposure — that is the deliberate trade: fewer bots means a longer run but slower cash.

---

## 6. The Sweep (Prestige)

When Exposure hits the ceiling, the operation **schedules a sweep** — it does not fire immediately. The player gets a red alert, one last chance to sell the staging pile, and an explicit *face the sweep* action. Only then do accounts roll a ban chance weighted by their heat.

This is not a punishment for one bad decision — it's the tide going out. But it must never rob someone who was asleep: the sweep waits at the door until the player opens it. (Playtest finding: the first build fired sweeps during offline catch-up and wiped 475,000 staged gold the player never had a chance to sell. Never again.)

Ban chance per account: `heat / 140`, zero below 40 heat.

**What's lost:** accounts, staged (unsold) gold, slots.
**What's kept:** all cash already sold, Contacts, tier. *(Laundered-vs-clean cash is planned; right now selling is itself the laundering step.)*

### Contacts (prestige currency)

```
contacts = floor( (run_cash / 20) ^ e )        // zero below a 45-minute run
e        = max(0.33, 0.5 - 0.03 × tier)
```

Run-scoped, not lifetime: each run pays for itself, so the number on the sweep screen is exactly what *this* operation earned. Early runs use a square root (generous — the first run pays ~6). As tiers stack the exponent slides toward a cube root, which keeps 1–3 day runs meaningful without letting cash explode the tree.

Contacts currently buy a flat **+12% yield each, permanent**. The tree below is *(planned)*:

- **Insider** — ban warnings earlier, or the sweep schedule itself.
- **Launderer** — better conversion of cash through a ban.
- **Script library** — automation presets so a new run sets itself up.
- **Second shard access** — a whole new market.
- **Front business** — passive clean income, no heat.

### Prestige anti-pattern guards

- **No reason to reset instantly.** Contacts scale with run cash, so holding a run longer always pays more. You can delay the sweep indefinitely; you just can't grow past the ceiling.
- **Run-length floor.** Under 45 minutes, prestige pays zero. No "reset every 10 minutes" spiral.
- **Setup persists.** *(planned)* Saved loadouts so a new run is "load preset, go" rather than ten minutes of clicking.
- **Prestige is a power fantasy.** After a reset you start with a bot and a cash float, not with the manual phase.

---

## 7. Progression Map

Each step unlocks a **mechanic**, not a bigger number. Times are measured from scripted playtests, first run.

| # | Unlock | Time | What it teaches |
|---|---|---|---|
| 1 | Manual farming — click to earn | 0:00 | Input → number |
| 2 | Sell gold for cash | 0:01 | Conversion is the real game |
| 3 | First bot — automation | 0:02 | Automation beats clicking (~19 clicks) |
| 4 | More accounts — width costs cash | 0:05 | Scaling |
| 5 | First heat warning (flagged) | 0:50 | Risk exists |
| 6 | Rack space — more slots | 1:00 | The tier gate |
| 7 | Proxies / humanizer — stealth | 1:20 | Buying safety, not throughput |
| 8 | Watched → imminent → terminated | 2:00 | Losing an account is normal |
| 9 | Sweep + Contacts | 3:40 | The reset is the reward |
| 10 | New identity, permanent multiplier | 3:45 | Meta-progression |
| 11 | Second shard — price arbitrage | *(planned)* | Markets are local |
| 12 | Buyer orders — bulk contracts | *(planned)* | Reputation |
| 13 | Rival bot farm / cartel | *(planned)* | Competition for zones |
| 14 | Front business / laundering | *(planned)* | Heat-free income |
| 15 | GM insider / arms race | *(planned)* | You're now the establishment |

The first run lands at ~3.5 hours of wall clock with maybe 25–40 minutes of actual attention. That's the first "oh no" — and it's winnable: the game warns you, you cash out, you learn what a wipe costs.

---

## 8. Interface

Text is not a limitation here — it's the right first build. A bot farm is a **log**. The most atmospheric thing you can show is a scrolling feed of your operation doing crimes.

### Screen (as shipped)

```
┌──────────────────────────────────────────────────────────────┐
│ IDLE BOT EMPIRE · SHARD 1        DAY 1 · 03:12 · RUN 3H 07M  │
├──────────────────────────────────────────────────────────────┤
│ GOLD STAGED       CASH                IDENTITY               │
│ 412K              $1,240.55           T0                     │
│ +150K/hr · 6/6    market $0.0020/g    0 contacts · 0 runs    │
├──────────────────────────────────────────────────────────────┤
│ EXPOSURE 78%                          NEXT SWEEP ~1H 12M     │
│ ████████████████░░░░░                                        │
├──────────────────────────────────────────────────────────────┤
│ [1] Acquire account  $26.20   [2] Add rack space    $120.00  │
│ [3] Sell 412K gold  $816.00   [4] Script v3         $152.10  │
│ [5] Proxy pool t2   $176.00   [6] Humanizer t1      $140.00  │
│ [   ] Less aggressive 0.9x    [   ] More aggressive 1.1x     │
├──────────────────────────────────────────────────────────────┤
│ ACCOUNTS — 6/6 SLOTS                                         │
│ #A01 ████████░░ watched  lv3   81K g/hr    [pause] [burn]    │
│ #A02 ████░░░░░░ flagged  lv2   64K g/hr    [pause] [burn]    │
├──────────────────────────────────────────────────────────────┤
│ 03:11 [!] #A01 is being watched (heat 72). Est. action ...   │
│ 03:12 [!!] Sweep incoming. Sell what you can, then face it.  │
└──────────────────────────────────────────────────────────────┘
```

Every figure above is the real thing: the sell button quotes the payout, the aggression buttons quote the value they set, and the exposure row is a countdown rather than a surprise.

### Interaction rules

- **Numbers auto-tick.** Commands are decisions, not actions. No "press enter to earn."
- **Decisions queue, they don't expire.** A queued sweep waits for you. Being away never costs you a decision.
- **Return screen is a scene.** *"While you were away 7h 12m: 100K gold, $500 on hand."*
- **Dev fast-forward** — `window.ibe.ff(seconds)` in the dev build only. Never a player-facing resource.
- **`why` on everything.** `why A17` itemises heat sources: `+0.9/hr crowding · +0.4 aggression · −0.7 residential proxy`. *(planned — the core anti-wiki promise, not built)*

### Architecture — Rust core, WASM, Preact shell

The simulation is **pure and renderer-agnostic**:

```rust
game.tick(delta_seconds);          // pure, deterministic, seeded RNG
let view = game.view();            // flat, render-ready
```

```
idle-bot-empire/
├── crates/
│   ├── sim/            # zero-I/O: state, advance, heat, sweep, contacts
│   │   └── balance.toml
│   └── wasm/           # thin wasm-bindgen boundary
└── web/                # Vite + Preact + TypeScript. Dumb view layer.
```

**Non-negotiables**

- **`sim` has no I/O and no clock.** JS passes elapsed milliseconds in. This is what makes offline catch-up identical to live play — one code path, no drifting second formula.
- **Seed the RNG in state.** `getrandom` doesn't exist in the browser; PCG32 with its state inside the save means a reload replays identically and a bug is reproducible from a save file.
- **Return a `ViewModel`, not the whole state.** ~40 flat fields. The UI never touches `State`.
- **Fixed timestep with an accumulator.** Whole 1s ticks; a 12h catch-up is 43,200 cheap iterations.
- **Save = versioned JSON via serde.** `version` and a migration fn from day one; the state shape will change.
- **Log rows are keyed by `seq`.** Stable ids mean the tick updates in place instead of rebuilding the list and throwing the scroll position away.

**Why this stack pays off:** balance is a `cargo test` that simulates hours of play and asserts pacing bands. You cannot write that against a UI-coupled loop, and the UI can be re-skinned without touching a rule.

---

## 9. Numbers

Authoritative: `crates/sim/balance.toml`. This table is a snapshot for reading convenience.

| Thing | Shipped value |
|---|---|
| Starting cash | $50 |
| Manual click | 600 g — the first bot takes ~19 clicks |
| Account cost | $22, ×1.06 per account owned |
| Account yield | 25,000 g/hr at level 1, ×1.08 per level, +1 level / 2h, max 40 |
| Slots | 5 base, $120, ×1.9 per slot |
| Script upgrade | $90, ×1.30, +12% yield/level |
| Proxy / humanizer | $110 / $140, ×1.6 / ×1.7, +0.28 / +0.22 stealth |
| Heat | 12/hr base, +6% per active account, `× aggression^1.7`, `÷ stealth` |
| Heat decay | 30/hr while paused, after a 10-min delay |
| Aggression | 0.2–2.0; yield `× a^1.0`, heat `× a^1.7` |
| Exposure | 5.5 per account-hour, `× aggression^1.2`, never decays |
| Run ceiling | `100 × 1.3^tier` |
| Market | $0.002/g, ±0.4%/hr walk, 2M g/hr turnover, −2% price per turnover dumped |
| Sweep ban chance | `heat / 140`, zero below 40 |
| Contacts | `floor((run_cash / 20) ^ max(0.33, 0.5 − 0.03 × tier))`, zero under 45 min |
| Contacts multiplier | +12% yield per contact, permanent |
| Offline cap | 12h at 100%, **never sold** |
| Measured first run | ~3.7h, 5–6 accounts, 2 banned, **+6 contacts** |

**The metric to tune against:** time-to-next-meaningful-purchase. Above ~3 minutes early or ~20 minutes mid-run and the curve is wrong. The `pacing` test guards the early band.

---

## 10. Anti-Patterns (Explicitly Refused)

| Don't | Do instead |
|---|---|
| Cap offline progress at 1–2h | 12h at full rate, extended by upgrade, shown in the UI |
| Let a player brick their run | Every account can be burned for salvage; a personal account always exists |
| Wipe setup on prestige | *(planned)* Saved loadouts; auto-reassign |
| Hide upgrade effects | `+12% gold/hr · +3 heat/hr` on every purchase, always |
| Make prestige a pure multiplier | *(target)* Every prestige tier should unlock a mechanic |
| Punish holding a run longer | Run-scoped Contacts with a sublinear-but-growing curve |
| Require a wiki | *(planned)* `why` on everything, in-game codex |
| Costs of 500% for +10% | Prestige gains feel like flying afterwards |
| Ads / energy / paid offline | Free build; one-time purchase at most |
| Instant bans from nowhere | Warning tiers, a visible countdown, and a recovery action |

---

## 11. Tone

Dry, deadpan, bureaucratic. The comedy is in the **gap between the log lines and what they mean**: a cheerful line about "restocking accounts" next to a line about a 19-year-old in a warehouse. Think *Papers, Please* meets a sysadmin dashboard.

Rules:

- **No real games.** The MMO is fictional ("*Kingdoms of Aetheria*" — placeholder). Gold is fictional. No real RMT, ever.
- **The victims are never shown.** The joke is the system, not the people in it. Punch up at the platforms and the market, not down at farmers.
- **The game never winks too hard.** One good joke per screen. The numbers do the work.
- Optional late-game tonal shift (the "world fights back" arc) is the genre-proven retention spike — Cookie Clicker's Grandmapocalypse. Time-box it, make it reversible, never in the first week.

---

## 12. Scope

**Shipped:** manual phase, one shard, one zone, accounts (buy/level/heat/flag/terminate/burn), selling with dump penalty, slots, script/proxy/humanizer upgrades, queued sweep, Contacts, offline catch-up, text UI, balance tests.

**Deliberately not yet:** graphics, multiple shards, buyer orders, rivals, laundering front, aged-account broker, milestone multipliers, Contact tree, `why` panel, saved loadouts, narrative events, sound, achievements.

**Cut line:** if it doesn't appear in the screen above, it isn't in the slice.

---

## 13. Decisions (settled 2026-10-27)

| # | Question | Decision | Consequence |
|---|---|---|---|
| 1 | Run ambition | **Hybrid** — first run ~3h, later runs stretch to 1–3 days | Ceiling grows `×1.3^tier`; Contacts exponent slides toward a cube root |
| 2 | Time model | **Real-time ticks + 12h offline catch-up** at full rate | One `advance()` path for live and offline; decisions never expire |
| 3 | Platform | **Web first**, retro-terminal aesthetic; desktop via Tauri later | Text UI is the shipped v1; graphics is a view swap |
| 4 | Stack | **Rust core → WASM + Preact/TS UI** | `sim` is I/O- and clock-free; balance is a `cargo test` |
| 5 | Monetization | **Free hobby build**, one-time purchase considered later | No ads/energy/paid offline |
| 6 | Tone | **Deadpan bureaucratic satire** | Log lines carry the joke; one gag per screen |

### Standing constraints

- Every mechanic must be expressible as pure state math — if it needs a timer or a DOM event, it belongs in the view layer.
- Nothing player-facing may be sold or gated that the design refuses (offline time, speed, ads).
- The run ceiling must be *visible* — the player always sees `next sweep: ~6h`, never a surprise wipe.

---

## 14. What Playtesting Changed

Four rules came out of running the sim, not from thinking about it:

1. **Sweeps queue instead of firing.** The first build ran the sweep inside offline catch-up and wiped 475,000 staged gold the player never got to sell. Now the ceiling schedules a sweep and `face_sweep` is explicit. Offline time can never rob a decision.
2. **Heat decay had to be 30/hr, not 8/hr.** At 8/hr a flagged account was out of action for ten hours — rotation was impossible, so the only strategy was "stop playing."
3. **The ceiling must grow slower than income.** At `×2.2` per tier, run two was *six hours* against a three-hour first run, because the ceiling doubled while the account count reset. `×1.3` fixed it.
4. **Early costs must stay under ~45 minutes of income.** A $260 slot after four $50/hr accounts produced hour-long dead spots with nothing affordable. Slots at $120 fixed it, and the pacing test now guards it.

### Next

- `why` — itemised heat sources per account. The core anti-wiki promise.
- Mule contract — auto-sell at a worse price so an idle player converts staging without checking in.
- Contact tree as an actual tree, with mechanics behind the nodes.

---

## Sources

- r/incremental_games — [What makes an idle game fun?](https://www.reddit.com/r/incremental_games/comments/1epowcl/what_makes_an_idle_game_fun/) (2024)
- r/incremental_games — [What features you DON'T like](https://www.reddit.com/r/incremental_games/comments/rgsw0j/what_features_you_dont_like_in_incrementalidle/) (2021)
- r/incremental_games — [Prestige mechanics often get it wrong](https://www.reddit.com/r/incremental_games/comments/7rp83l/where_i_think_prestige_mechanics_often_get_it/) (2018)
- r/incremental_games — [Session times & off-time](https://www.reddit.com/r/incremental_games/comments/ok7gtn/discussion_session_times_session_lengths_and/) (2021)
- r/incremental_games — [Unfolding incrementals](https://www.reddit.com/r/incremental_games/comments/1p2zr94/what_do_you_like_or_dislike_about_unfolding/) (2025)
- Anthony Pecorella — [The Math of Idle Games](https://www.gamedeveloper.com/design/the-math-of-idle-games-part-iii), GDC Europe 2016
- [Idle and Incremental Game Design: The Design of Waiting](https://gamedesign.gg/articles/idle-and-incremental-game-design/) — pacing, offline, prestige
- [Cookie Clicker systems analysis](https://teemo.dev/game-design/cookie-clicker/design/overview-notes/) — two exponentials, interval tuning
- ["It Started as a Joke": On the Design of Idle Games](https://www.researchgate.net/publication/336711769_It_Started_as_a_Joke_On_the_Design_of_Idle_Games), CHI PLAY 2019
- [Trailer Park Boys: Greasy Money designer diary](https://www.gamedeveloper.com/design/designer-diary-trailer-park-boys-greasy-money) — prestige as narrative
- Far Cry 6 [suspicion/aggression system](https://news.ubisoft.com/en-au/article/1coXQaIzFcbrESiapwUQJK/improving-stealth-in-far-cry-6) + [stealth detection design](https://xesoft.dev/guides/unity-stealth-suspicion-meter/) — heat meter tiers, decay delay, no binary detection
