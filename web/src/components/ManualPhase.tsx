import { formatCompactNumber, formatMoney } from '../format';
import type { ViewModel } from '../types';
import { Bar } from './Bar';

interface ManualPhaseProps {
  view: ViewModel;
  exec: (command: string) => void;
}

export function ManualPhase({ view, exec }: ManualPhaseProps) {
  const percent = (view.gold / Math.max(1, view.first_bot_gold)) * 100;

  return (
    <div class="manual">
      <button type="button" class="manual-btn" onClick={() => exec('click')}>
        <span class="key">1</span>
        <span class="label">Farm by hand</span>
        <span class="cost">+{formatCompactNumber(view.click_gold)} g</span>
      </button>
      <div class="manual-progress">
        <div class="row">
          <span>
            First bot: {formatCompactNumber(view.gold)} /{' '}
            {formatCompactNumber(view.first_bot_gold)} g
          </span>
          <span>{Math.min(100, Math.floor(percent))}%</span>
        </div>
        <Bar percent={percent} />
      </div>
      <button
        type="button"
        class="cmd"
        onClick={() => exec('sell')}
        disabled={view.gold < 1}
      >
        <span class="key">2</span>
        <span>Sell {formatCompactNumber(view.gold)} gold</span>
        <span class="cost">{formatMoney(view.sell_proceeds)}</span>
      </button>
      <button
        type="button"
        class="cmd"
        onClick={() => exec('buy_account')}
        disabled={view.cash < view.costs.account}
      >
        <span class="key">3</span>
        <span>Buy your first bot</span>
        <span class="cost">{formatMoney(view.costs.account)}</span>
      </button>
      <p class="hint">
        No bots yet. Farm by hand, sell the gold, hire the bot. After that you
        only do this for fun.
      </p>
    </div>
  );
}
