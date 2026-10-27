import { formatCompactNumber, formatMoney } from '../format';
import type { ViewModel } from '../types';

interface CommandsProps {
  view: ViewModel;
  exec: (command: string) => void;
}

interface CommandItem {
  key: string;
  command: string;
  label: string;
  cost: string;
  enabled: boolean;
}

export function Commands({ view, exec }: CommandsProps) {
  const items: CommandItem[] = [
    {
      key: '1',
      command: 'buy_account',
      label: 'Acquire account',
      cost: formatMoney(view.costs.account),
      enabled:
        view.cash >= view.costs.account && view.accounts.length < view.capacity,
    },
    {
      key: '2',
      command: 'buy_slot',
      label: 'Add rack space',
      cost: formatMoney(view.costs.slot),
      enabled:
        view.cash >= view.costs.slot && view.accounts.length >= view.capacity,
    },
    {
      key: '3',
      command: 'sell',
      label: `Sell ${formatCompactNumber(view.gold)} gold`,
      cost: formatMoney(view.sell_proceeds),
      enabled: view.gold >= 1,
    },
    {
      key: '4',
      command: 'upgrade:script',
      label: `Script v${view.script_level + 2}`,
      cost: formatMoney(view.costs.script),
      enabled: view.cash >= view.costs.script,
    },
    {
      key: '5',
      command: 'upgrade:proxy',
      label: `Proxy pool t${view.proxy_level + 1}`,
      cost: formatMoney(view.costs.proxy),
      enabled: view.cash >= view.costs.proxy,
    },
    {
      key: '6',
      command: 'upgrade:humanizer',
      label: `Humanizer t${view.humanizer_level + 1}`,
      cost: formatMoney(view.costs.humanizer),
      enabled: view.cash >= view.costs.humanizer,
    },
  ];

  const lower = Math.max(0.2, Math.round((view.aggression - 0.1) * 10) / 10);
  const higher = Math.min(2, Math.round((view.aggression + 0.1) * 10) / 10);

  return (
    <>
      <div class="cmdgrid">
        {items.map((item) => (
          <button
            type="button"
            key={item.command}
            class="cmd"
            disabled={!item.enabled || view.over}
            onClick={() => exec(item.command)}
          >
            <span class="key">{item.key}</span>
            <span>{item.label}</span>
            <span class="cost">{item.cost}</span>
          </button>
        ))}
      </div>
      <div class="cmdgrid">
        <button
          type="button"
          class="cmd"
          disabled={view.over || view.aggression <= 0.2}
          onClick={() => exec(`aggression:${lower}`)}
        >
          <span class="key">[</span>
          <span>Less aggressive</span>
          <span class="cost">{lower.toFixed(1)}x</span>
        </button>
        <button
          type="button"
          class="cmd"
          disabled={view.over || view.aggression >= 2}
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
