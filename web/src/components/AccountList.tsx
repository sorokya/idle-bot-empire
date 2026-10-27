import { formatCompactNumber } from '../format';
import type { AccountView, ViewModel } from '../types';

interface AccountListProps {
  view: ViewModel;
  exec: (command: string) => void;
}

function heatTone(account: AccountView): string {
  if (account.band === 'quiet') return '';
  if (account.band === 'imminent') return 'danger';
  return 'warn';
}

export function AccountList({ view, exec }: AccountListProps) {
  return (
    <div class="section">
      <h2>
        Accounts — {view.accounts.length}/{view.capacity} slots
      </h2>
      {view.accounts.length === 0 && (
        <div class="acct muted">No accounts. You are the bot.</div>
      )}
      {view.accounts.map((account) => {
        const tone = heatTone(account);
        return (
          <div class="acct" key={account.id}>
            <span class="tag">{account.tag}</span>
            <span class={`heat ${tone}`}>
              <i style={{ width: `${Math.min(100, account.heat)}%` }} />
            </span>
            <span class={`band ${tone}`}>{account.band}</span>
            <span>lv{account.level}</span>
            <span class="yield">
              {account.paused
                ? '— paused —'
                : `${formatCompactNumber(account.yield_per_hr)} g/hr`}
            </span>
            <button type="button" onClick={() => exec(`pause:${account.id}`)}>
              {account.paused ? 'resume' : 'pause'}
            </button>
            <button type="button" onClick={() => exec(`burn:${account.id}`)}>
              burn
            </button>
          </div>
        );
      })}
    </div>
  );
}
