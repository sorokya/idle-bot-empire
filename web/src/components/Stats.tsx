import { formatCompactNumber, formatDuration, formatMoney } from '../format';
import type { ViewModel } from '../types';
import { Bar } from './Bar';

interface StatsProps {
  view: ViewModel;
}

function exposureTone(ratio: number): string {
  if (ratio > 0.8) return 'danger';
  if (ratio > 0.55) return 'warn';
  return '';
}

function sweepLabel(view: ViewModel): string {
  if (view.sweep_pending) return 'sweep imminent';
  if (view.sweep_eta_hours == null) return 'no sweep';
  return `next sweep ~${formatDuration(view.sweep_eta_hours * 3600)}`;
}

export function Stats({ view }: StatsProps) {
  const exposureRatio = view.exposure / view.ceiling;

  return (
    <>
      <div class="stats">
        <div class="stat">
          <div class="label">Gold staged</div>
          <div class="value">{formatCompactNumber(view.gold)}</div>
          <div class="sub">
            +{formatCompactNumber(view.gold_per_hr)}/hr · {view.active}/
            {view.capacity} accounts active
          </div>
        </div>
        <div class="stat">
          <div class="label">Cash</div>
          <div class="value money">{formatMoney(view.cash)}</div>
          <div class="sub">market ${view.market_price.toFixed(4)}/g</div>
        </div>
        <div class="stat">
          <div class="label">Identity</div>
          <div class="value">T{view.tier}</div>
          <div class="sub">
            {view.contacts} contacts · {view.runs_completed} runs
          </div>
        </div>
      </div>
      <div class="exposure">
        <div class="row">
          <span>Exposure {Math.floor(exposureRatio * 100)}%</span>
          <span>{sweepLabel(view)}</span>
        </div>
        <Bar percent={exposureRatio * 100} tone={exposureTone(exposureRatio)} />
      </div>
    </>
  );
}
