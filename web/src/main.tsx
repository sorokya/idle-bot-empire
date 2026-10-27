import { render } from 'preact';
import { useCallback, useEffect, useRef, useState } from 'preact/hooks';
import init, { IbeGame } from './wasm/ibe_wasm.js';
import './style.css';

interface AccountView {
  id: number;
  tag: string;
  level: number;
  heat: number;
  band: string;
  paused: boolean;
  age_hrs: number;
  yield_per_hr: number;
}

interface LogView {
  seq: number;
  clock: string;
  kind: string;
  text: string;
}

interface Costs {
  account: number;
  slot: number;
  script: number;
  proxy: number;
  humanizer: number;
}

interface View {
  day: number;
  clock: string;
  run_hrs: number;
  offline_secs: number;
  gold: number;
  gold_per_hr: number;
  cash: number;
  sell_proceeds: number;
  accounts: AccountView[];
  active: number;
  capacity: number;
  exposure: number;
  ceiling: number;
  sweep_eta_hours: number | null;
  market_price: number;
  aggression: number;
  script_level: number;
  proxy_level: number;
  humanizer_level: number;
  costs: Costs;
  tier: number;
  contacts: number;
  pending_contacts: number;
  runs_completed: number;
  banned_total: number;
  over: boolean;
  sweep_pending: boolean;
  started: boolean;
  click_gold: number;
  first_bot_gold: number;
  log: LogView[];
}

const SAVE_KEY = 'idle-bot-empire/save/v1';

// ---------------------------------------------------------------- helpers

function n(input: number): string {
  let v = input;
  if (!Number.isFinite(v)) return '0';
  if (v < 1000) return Math.floor(v).toString();
  const units = ['', 'K', 'M', 'B', 'T', 'Qa', 'Qi', 'Sx'];
  let i = 0;
  while (v >= 1000 && i < units.length - 1) {
    v /= 1000;
    i++;
  }
  const dec = v < 10 ? 2 : v < 100 ? 1 : 0;
  return v.toFixed(dec) + units[i];
}

function money(v: number): string {
  return (
    '$' +
    v.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
  );
}

function dur(secs: number): string {
  const s = Math.max(0, Math.floor(secs));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

let suppressSave = false;

function persist(game: IbeGame) {
  if (suppressSave) return;
  try {
    localStorage.setItem(SAVE_KEY, game.save(Date.now() / 1000));
  } catch {
    /* quota */
  }
}

async function boot(): Promise<{ game: IbeGame; view: View; away: number }> {
  await init();
  const raw = localStorage.getItem(SAVE_KEY);
  let game: IbeGame;
  if (raw) {
    try {
      game = IbeGame.loadSave(raw, Date.now() / 1000);
    } catch (e) {
      console.warn('save failed to load, starting fresh', e);
      game = new IbeGame(Date.now() % 1_000_000);
    }
  } else {
    game = new IbeGame(Date.now() % 1_000_000);
  }
  const view = game.view() as View;
  return { game, view, away: view.offline_secs ?? 0 };
}

// ------------------------------------------------------------------- hook

function useGame() {
  const [view, setView] = useState<View | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [away, setAway] = useState(0);
  const gameRef = useRef<IbeGame | null>(null);

  useEffect(() => {
    let cancelled = false;
    let tickId = 0;
    let saveId = 0;

    void boot().then(({ game, view: first, away: off }) => {
      if (cancelled) return;
      gameRef.current = game;
      setView(first);
      setAway(off);

      let last = Date.now();
      tickId = window.setInterval(() => {
        const now = Date.now();
        game.tick(now - last);
        last = now;
        setView(game.view() as View);
      }, 250);
      saveId = window.setInterval(() => persist(game), 5000);
      window.addEventListener('pagehide', () => persist(game));

      if (import.meta.env.DEV) {
        (window as unknown as { ibe: unknown }).ibe = {
          ff: (secs: number) => {
            game.tick(secs * 1000);
            setView(game.view() as View);
            return game.view();
          },
          view: () => game.view(),
          reset: () => {
            suppressSave = true;
            localStorage.removeItem(SAVE_KEY);
            location.reload();
          },
        };
      }
    });

    return () => {
      cancelled = true;
      window.clearInterval(tickId);
      window.clearInterval(saveId);
    };
  }, []);

  const exec = useCallback((cmd: string) => {
    const game = gameRef.current;
    if (!game) return;
    const err = game.dispatch(cmd);
    setError(err ?? null);
    setView(game.view() as View);
  }, []);

  return { view, exec, error, away };
}

// ------------------------------------------------------------- components

function Bar({ pct, tone = '' }: { pct: number; tone?: string }) {
  return (
    <div class={`bar ${tone}`}>
      <i style={{ width: `${Math.min(100, Math.max(0, pct))}%` }} />
    </div>
  );
}

function TitleBar({ v }: { v: View }) {
  return (
    <div class="titlebar">
      <span>
        <b>Idle Bot Empire</b> · shard 1
      </span>
      <span>
        day {v.day} · {v.clock} · run {dur(v.run_hrs * 3600)}
      </span>
    </div>
  );
}

function Stats({ v }: { v: View }) {
  return (
    <>
      <div class="stats">
        <div class="stat">
          <div class="label">Gold staged</div>
          <div class="value">{n(v.gold)}</div>
          <div class="sub">
            +{n(v.gold_per_hr)}/hr · {v.active}/{v.capacity} accounts active
          </div>
        </div>
        <div class="stat">
          <div class="label">Cash</div>
          <div class="value money">{money(v.cash)}</div>
          <div class="sub">market ${v.market_price.toFixed(4)}/g</div>
        </div>
        <div class="stat">
          <div class="label">Identity</div>
          <div class="value">T{v.tier}</div>
          <div class="sub">
            {v.contacts} contacts · {v.runs_completed} runs
          </div>
        </div>
      </div>
      <div class="exposure">
        <div class="row">
          <span>Exposure {Math.floor((v.exposure / v.ceiling) * 100)}%</span>
          <span>
            {v.sweep_pending
              ? 'sweep imminent'
              : v.sweep_eta_hours == null
                ? 'no sweep'
                : `next sweep ~${dur(v.sweep_eta_hours * 3600)}`}
          </span>
        </div>
        <Bar
          pct={(v.exposure / v.ceiling) * 100}
          tone={
            v.exposure / v.ceiling > 0.8
              ? 'danger'
              : v.exposure / v.ceiling > 0.55
                ? 'warn'
                : ''
          }
        />
      </div>
    </>
  );
}

function ManualPhase({ v, exec }: { v: View; exec: (cmd: string) => void }) {
  const pct = (v.gold / Math.max(1, v.first_bot_gold)) * 100;
  return (
    <div class="manual">
      <button type="button" class="manual-btn" onClick={() => exec('click')}>
        <span class="key">1</span>
        <span class="label">Farm by hand</span>
        <span class="cost">+{n(v.click_gold)} g</span>
      </button>
      <div class="manual-progress">
        <div class="row">
          <span>
            First bot: {n(v.gold)} / {n(v.first_bot_gold)} g
          </span>
          <span>{Math.min(100, Math.floor(pct))}%</span>
        </div>
        <Bar pct={pct} />
      </div>
      <button
        type="button"
        class="cmd"
        onClick={() => exec('sell')}
        disabled={v.gold < 1}
      >
        <span class="key">2</span>
        <span>Sell {n(v.gold)} gold</span>
        <span class="cost">{money(v.sell_proceeds)}</span>
      </button>
      <button
        type="button"
        class="cmd"
        onClick={() => exec('buy_account')}
        disabled={v.cash < v.costs.account}
      >
        <span class="key">3</span>
        <span>Buy your first bot</span>
        <span class="cost">{money(v.costs.account)}</span>
      </button>
      <p class="hint">
        No bots yet. Farm by hand, sell the gold, hire the bot. After that you
        only do this for fun.
      </p>
    </div>
  );
}

function Commands({ v, exec }: { v: View; exec: (cmd: string) => void }) {
  const items = [
    {
      key: '1',
      cmd: 'buy_account',
      label: 'Acquire account',
      cost: money(v.costs.account),
      enabled: v.cash >= v.costs.account && v.accounts.length < v.capacity,
    },
    {
      key: '2',
      cmd: 'buy_slot',
      label: 'Add rack space',
      cost: money(v.costs.slot),
      enabled: v.cash >= v.costs.slot && v.accounts.length >= v.capacity,
    },
    {
      key: '3',
      cmd: 'sell',
      label: `Sell ${n(v.gold)} gold`,
      cost: money(v.sell_proceeds),
      enabled: v.gold >= 1,
    },
    {
      key: '4',
      cmd: 'upgrade:script',
      label: `Script v${v.script_level + 2}`,
      cost: money(v.costs.script),
      enabled: v.cash >= v.costs.script,
    },
    {
      key: '5',
      cmd: 'upgrade:proxy',
      label: `Proxy pool t${v.proxy_level + 1}`,
      cost: money(v.costs.proxy),
      enabled: v.cash >= v.costs.proxy,
    },
    {
      key: '6',
      cmd: 'upgrade:humanizer',
      label: `Humanizer t${v.humanizer_level + 1}`,
      cost: money(v.costs.humanizer),
      enabled: v.cash >= v.costs.humanizer,
    },
  ];

  const lower = Math.max(0.2, Math.round((v.aggression - 0.1) * 10) / 10);
  const higher = Math.min(2, Math.round((v.aggression + 0.1) * 10) / 10);

  return (
    <>
      <div class="cmdgrid">
        {items.map((it) => (
          <button
            type="button"
            key={it.cmd}
            class="cmd"
            disabled={!it.enabled || v.over}
            onClick={() => exec(it.cmd)}
          >
            <span class="key">{it.key}</span>
            <span>{it.label}</span>
            <span class="cost">{it.cost}</span>
          </button>
        ))}
      </div>
      <div class="cmdgrid">
        <button
          type="button"
          class="cmd"
          disabled={v.over || v.aggression <= 0.2}
          onClick={() => exec(`aggression:${lower}`)}
        >
          <span class="key">[</span>
          <span>Less aggressive</span>
          <span class="cost">{lower.toFixed(1)}x</span>
        </button>
        <button
          type="button"
          class="cmd"
          disabled={v.over || v.aggression >= 2}
          onClick={() => exec(`aggression:${higher}`)}
        >
          <span class="key">]</span>
          <span>More aggressive</span>
          <span class="cost">{higher.toFixed(1)}x</span>
        </button>
      </div>
    </>
  );
}

function AccountList({ v, exec }: { v: View; exec: (cmd: string) => void }) {
  return (
    <div class="section">
      <h2>
        Accounts — {v.accounts.length}/{v.capacity} slots
      </h2>
      {v.accounts.length === 0 && (
        <div class="acct muted">No accounts. You are the bot.</div>
      )}
      {v.accounts.map((a) => {
        const tone =
          a.band === 'quiet' ? '' : a.band === 'imminent' ? 'danger' : 'warn';
        return (
          <div class="acct" key={a.id}>
            <span class="tag">{a.tag}</span>
            <span class={`heat ${tone}`}>
              <i style={{ width: `${Math.min(100, a.heat)}%` }} />
            </span>
            <span class={`band ${tone}`}>{a.band}</span>
            <span>lv{a.level}</span>
            <span class="yield">
              {a.paused ? '— paused —' : `${n(a.yield_per_hr)} g/hr`}
            </span>
            <button type="button" onClick={() => exec(`pause:${a.id}`)}>
              {a.paused ? 'resume' : 'pause'}
            </button>
            <button type="button" onClick={() => exec(`burn:${a.id}`)}>
              burn
            </button>
          </div>
        );
      })}
    </div>
  );
}

function LogPane({ v }: { v: View }) {
  const ref = useRef<HTMLDivElement>(null);
  const atBottom = useRef(true);

  useEffect(() => {
    const el = ref.current;
    if (!el || !atBottom.current) return;
    el.scrollTop = el.scrollHeight;
  }, [v.log.length]);

  return (
    <div
      class="log"
      ref={ref}
      onScroll={(e) => {
        const el = e.currentTarget as HTMLDivElement;
        atBottom.current =
          el.scrollHeight - el.scrollTop - el.clientHeight < 24;
      }}
    >
      {v.log.map((l) => (
        <div key={l.seq} class={`kind-${l.kind.toLowerCase()}`}>
          <span class="t">{l.clock}</span>
          <span class="m">{l.text}</span>
        </div>
      ))}
    </div>
  );
}

function Modals({ v, exec }: { v: View; exec: (cmd: string) => void }) {
  if (v.over) {
    return (
      <div class="overlay">
        <div class="modal">
          <h3>Run terminated</h3>
          <p>
            The shard came back up. Your accounts did not. Everything staged is
            gone — the cash you already sold is yours.
          </p>
          <div class="big">+{v.pending_contacts} contacts</div>
          <p>Contacts are permanent. Spend them on being harder to catch.</p>
          <button type="button" onClick={() => exec('prestige')}>
            New identity
          </button>
        </div>
      </div>
    );
  }
  if (v.sweep_pending) {
    return (
      <div class="overlay">
        <div class="modal warn">
          <h3>Sweep imminent</h3>
          <p>
            Exposure is maxed. They are already looking. You get one last chance
            to sell.
          </p>
          <p>
            <b>{n(v.gold)} gold</b> is staged and will be seized if you face it
            now.
          </p>
          <div class="modal-actions">
            <button
              type="button"
              class="money"
              onClick={() => exec('sell')}
              disabled={v.gold < 1}
            >
              Sell for {money(v.sell_proceeds)}
            </button>
            <button
              type="button"
              class="danger"
              onClick={() => exec('face_sweep')}
            >
              Face the sweep
            </button>
          </div>
        </div>
      </div>
    );
  }
  return null;
}

function App() {
  const { view, exec, error, away } = useGame();
  const [toastMsg, setToastMsg] = useState<string | null>(null);

  useEffect(() => {
    if (error) setToastMsg(error);
  }, [error]);

  useEffect(() => {
    if (!toastMsg) return;
    const id = window.setTimeout(() => setToastMsg(null), 2600);
    return () => window.clearTimeout(id);
  }, [toastMsg]);

  useEffect(() => {
    if (!view || away < 60) return;
    setToastMsg(
      `While you were away ${dur(away)}: ${n(view.gold)} gold, ${money(view.cash)} on hand.`,
    );
  }, [view, away]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey || e.altKey || !view) return;
      const map: Record<string, string> = view.started
        ? {
            '1': 'buy_account',
            '2': 'buy_slot',
            '3': 'sell',
            '4': 'upgrade:script',
            '5': 'upgrade:proxy',
            '6': 'upgrade:humanizer',
          }
        : { '1': 'click', '2': 'sell', '3': 'buy_account' };
      const extra: Record<string, string> = {
        p: 'prestige',
        f: 'face_sweep',
        '[': `aggression:${Math.max(0.2, Math.round((view.aggression - 0.1) * 10) / 10)}`,
        ']': `aggression:${Math.min(2, Math.round((view.aggression + 0.1) * 10) / 10)}`,
      };
      const cmd = map[e.key.toLowerCase()] ?? extra[e.key.toLowerCase()];
      if (cmd) {
        e.preventDefault();
        exec(cmd);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [view, exec]);

  if (!view) {
    return (
      <div class="frame">
        <div class="section muted">booting…</div>
      </div>
    );
  }

  return (
    <>
      <div class="frame">
        <TitleBar v={view} />
        <Stats v={view} />
        {view.started ? (
          <Commands v={view} exec={exec} />
        ) : (
          <ManualPhase v={view} exec={exec} />
        )}
      </div>

      <div class="frame">
        <AccountList v={view} exec={exec} />
        <LogPane v={view} />
      </div>

      <div class="foot">
        1–6 act · [ ] greed · p new identity · {view.banned_total} banned
        all-time
      </div>

      <Modals v={view} exec={exec} />
      {toastMsg && <div class="toast show">{toastMsg}</div>}
    </>
  );
}

render(<App />, document.querySelector('#app')!);
