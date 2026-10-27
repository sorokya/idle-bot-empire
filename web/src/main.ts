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
  log: LogView[];
}

const SAVE_KEY = 'idle-bot-empire/save/v1';
const app = document.querySelector<HTMLDivElement>('#app')!;

let game: IbeGame;
let view: View;
let lastMs = Date.now();
let awaySecs = 0;
let suppressSave = false;

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

function esc(s: string): string {
  return s.replace(
    /[&<>"']/g,
    (c) =>
      ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[
        c
      ] as string,
  );
}

let toastTimer: number | undefined;
function toast(msg: string) {
  let el = document.querySelector<HTMLDivElement>('.toast');
  if (!el) {
    el = document.createElement('div');
    el.className = 'toast';
    document.body.appendChild(el);
  }
  el.textContent = msg;
  el.classList.add('show');
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => el?.classList.remove('show'), 2600);
}

// ------------------------------------------------------------------ boot

async function boot() {
  await init();

  const raw = localStorage.getItem(SAVE_KEY);
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

  view = game.view() as View;
  awaySecs = view.offline_secs ?? 0;

  lastMs = Date.now();
  window.setInterval(tick, 250);
  window.setInterval(save, 5000);
  window.addEventListener('pagehide', save);
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) save();
  });

  bindInput();
  render();

  // Dev-only console handle. `ibe.ff(3600)` fast-forwards an hour of sim time
  // through the exact same tick path an offline return uses.
  if (import.meta.env.DEV) {
    (window as unknown as { ibe: unknown }).ibe = {
      ff: (secs: number) => {
        game.tick(secs * 1000);
        view = game.view() as View;
        render();
        return view;
      },
      view: () => game.view(),
      reset: () => {
        suppressSave = true;
        localStorage.removeItem(SAVE_KEY);
        location.reload();
      },
    };
  }
}

function tick() {
  const now = Date.now();
  const dt = now - lastMs;
  lastMs = now;
  if (dt > 0) game.tick(dt);
  view = game.view() as View;
  render();
}

function save() {
  if (suppressSave) return;
  try {
    localStorage.setItem(SAVE_KEY, game.save(Date.now() / 1000));
  } catch {
    /* quota — ignore */
  }
}

function exec(cmd: string) {
  const err = game.dispatch(cmd);
  if (err) toast(err);
  view = game.view() as View;
  render();
}

// ----------------------------------------------------------------- input

function bindInput() {
  document.addEventListener('click', (e) => {
    const el = (e.target as HTMLElement).closest<HTMLElement>('[data-cmd]');
    if (!el || (el as HTMLButtonElement).disabled) return;
    exec(el.dataset.cmd!);
  });

  document.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const map: Record<string, string> = {
      '1': 'buy_account',
      '2': 'buy_slot',
      '3': 'sell',
      '4': 'upgrade:script',
      '5': 'upgrade:proxy',
      '6': 'upgrade:humanizer',
      p: 'prestige',
      f: 'face_sweep',
      '[': `aggression:${(view.aggression - 0.1).toFixed(1)}`,
      ']': `aggression:${(view.aggression + 0.1).toFixed(1)}`,
    };
    const cmd = map[e.key.toLowerCase()];
    if (cmd) {
      e.preventDefault();
      exec(cmd);
    }
  });
}

// ---------------------------------------------------------------- render

function render() {
  const v = view;
  const exposurePct = Math.min(100, (v.exposure / v.ceiling) * 100);
  const exposureClass =
    exposurePct > 80 ? 'danger' : exposurePct > 55 ? 'warn' : '';
  const eta = v.sweep_eta_hours;

  const cmds = [
    {
      key: '1',
      cmd: 'buy_account',
      label: 'Acquire account',
      cost: v.costs.account,
      enabled:
        v.cash >= v.costs.account && v.accounts.length < v.capacity && !v.over,
    },
    {
      key: '2',
      cmd: 'buy_slot',
      label: 'Add rack space',
      cost: v.costs.slot,
      enabled:
        v.cash >= v.costs.slot && v.accounts.length >= v.capacity && !v.over,
    },
    {
      key: '3',
      cmd: 'sell',
      label: `Sell ${n(v.gold)} gold @ $${v.market_price.toFixed(4)}`,
      cost: null as number | null,
      enabled: v.gold >= 1 && !v.over,
    },
    {
      key: '4',
      cmd: 'upgrade:script',
      label: `Script v${v.script_level + 2}`,
      cost: v.costs.script,
      enabled: v.cash >= v.costs.script && !v.over,
    },
    {
      key: '5',
      cmd: 'upgrade:proxy',
      label: `Proxy pool t${v.proxy_level + 1}`,
      cost: v.costs.proxy,
      enabled: v.cash >= v.costs.proxy && !v.over,
    },
    {
      key: '6',
      cmd: 'upgrade:humanizer',
      label: `Humanizer t${v.humanizer_level + 1}`,
      cost: v.costs.humanizer,
      enabled: v.cash >= v.costs.humanizer && !v.over,
    },
  ];

  const accounts = v.accounts
    .map((a) => {
      const bandClass =
        a.band === 'quiet' ? '' : a.band === 'imminent' ? 'danger' : 'warn';
      return `<div class="acct">
        <span class="tag">${esc(a.tag)}</span>
        <span class="heat ${bandClass}"><i style="width:${Math.min(100, a.heat)}%"></i></span>
        <span class="band ${bandClass}">${a.band}</span>
        <span>lv${a.level}</span>
        <span class="yield">${a.paused ? '— paused —' : `${n(a.yield_per_hr)} g/hr`}</span>
        <button data-cmd="pause:${a.id}">${a.paused ? 'resume' : 'pause'}</button>
        <button data-cmd="burn:${a.id}">burn</button>
      </div>`;
    })
    .join('');

  const log = v.log
    .map(
      (l) =>
        `<div class="kind-${l.kind.toLowerCase()}"><span class="t">${esc(l.clock)}</span><span class="m">${esc(l.text)}</span></div>`,
    )
    .join('');

  app.innerHTML = `
    <div class="frame">
      <div class="titlebar">
        <span><b>Idle Bot Empire</b> · shard 1</span>
        <span>day ${v.day} · ${v.clock} · run ${dur(v.run_hrs * 3600)}</span>
      </div>
      <div class="stats">
        <div class="stat">
          <div class="label">Gold staged</div>
          <div class="value">${n(v.gold)}</div>
          <div class="sub">+${n(v.gold_per_hr)}/hr · ${v.active}/${v.capacity} accounts active</div>
        </div>
        <div class="stat">
          <div class="label">Cash</div>
          <div class="value money">${money(v.cash)}</div>
          <div class="sub">market $${v.market_price.toFixed(4)}/g</div>
        </div>
        <div class="stat">
          <div class="label">Identity</div>
          <div class="value">T${v.tier}</div>
          <div class="sub">${v.contacts} contacts · ${v.runs_completed} runs</div>
        </div>
      </div>
      <div class="exposure">
        <div class="row">
          <span>Exposure ${Math.floor(exposurePct)}%</span>
          <span>${eta === null || eta === undefined ? 'sweep pending' : `next sweep ~${dur(eta * 3600)}`}</span>
        </div>
        <div class="bar ${exposureClass}"><i style="width:${exposurePct}%"></i></div>
      </div>
      <div class="cmdgrid">
        ${cmds
          .map(
            (
              c,
            ) => `<button class="cmd" data-cmd="${c.cmd}" ${c.enabled ? '' : 'disabled'}>
              <span class="key">${c.key}</span><span>${esc(c.label)}</span>
              ${c.cost === null ? '' : `<span class="cost">${money(c.cost)}</span>`}
            </button>`,
          )
          .join('')}
      </div>
      <div class="cmdgrid">
        <button class="cmd" data-cmd="aggression:${(v.aggression - 0.1).toFixed(1)}" ${v.over ? 'disabled' : ''}>
          <span class="key">[</span><span>Less aggressive</span><span class="cost">${v.aggression.toFixed(1)}x</span>
        </button>
        <button class="cmd" data-cmd="aggression:${(v.aggression + 0.1).toFixed(1)}" ${v.over ? 'disabled' : ''}>
          <span class="key">]</span><span>More aggressive</span><span class="cost">${v.aggression.toFixed(1)}x</span>
        </button>
      </div>
    </div>

    <div class="frame">
      <div class="section">
        <h2>Accounts — ${v.accounts.length}/${v.capacity} slots</h2>
        ${accounts || `<div class="acct"><span class="band">no accounts</span></div>`}
      </div>
      <div class="log">${log}</div>
    </div>

    <div class="foot">1–6 act · [ ] greed · p new identity · ${v.banned_total} banned all-time</div>
    ${overlay(v)}
  `;
}

function overlay(v: View): string {
  if (v.sweep_pending && !v.over) {
    return `<div class="overlay">
      <div class="modal" style="border-color:var(--warn)">
        <h3 style="color:var(--warn)">Sweep imminent</h3>
        <p>Exposure is maxed. They are already looking. You get one last chance to sell.</p>
        <p><b>${n(v.gold)} gold</b> is staged and will be seized if you face it now.</p>
        <button data-cmd="sell" style="border-color:var(--money);color:var(--money)">Sell ${n(v.gold)} gold</button>
        <button data-cmd="face_sweep" style="margin-left:8px;border-color:var(--danger);color:var(--danger)">Face the sweep</button>
      </div>
    </div>`;
  }
  if (!v.over) return '';
  return `<div class="overlay">
    <div class="modal">
      <h3>Run terminated</h3>
      <p>The shard came back up. Your accounts did not. Everything staged is gone — the cash you already sold is yours.</p>
      <div class="big">+${v.pending_contacts} contacts</div>
      <p>Contacts are permanent. Spend them on being harder to catch.</p>
      <button data-cmd="prestige">New identity</button>
    </div>
  </div>`;
}

boot().then(() => {
  if (awaySecs > 60) {
    toast(
      `While you were away ${dur(awaySecs)}: ${n(view.gold)} gold, ${money(view.cash)} on hand.`,
    );
    awaySecs = 0;
  }
});
