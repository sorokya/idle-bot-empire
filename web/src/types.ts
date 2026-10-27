export type HeatBand = 'quiet' | 'flagged' | 'watched' | 'imminent';

export type LogKind = 'Info' | 'Money' | 'Warn' | 'Danger' | 'System';

export interface AccountView {
  id: number;
  tag: string;
  level: number;
  heat: number;
  band: HeatBand;
  paused: boolean;
  age_hrs: number;
  yield_per_hr: number;
}

export interface LogView {
  seq: number;
  clock: string;
  kind: LogKind;
  text: string;
}

export interface Costs {
  account: number;
  slot: number;
  script: number;
  proxy: number;
  humanizer: number;
}

export interface ViewModel {
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
