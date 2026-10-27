import { formatCompactNumber, formatMoney } from '../format';
import type { ViewModel } from '../types';

interface ModalsProps {
  view: ViewModel;
  exec: (command: string) => void;
}

export function Modals({ view, exec }: ModalsProps) {
  if (view.over) {
    return (
      <div class="overlay">
        <div class="modal">
          <h3>Run terminated</h3>
          <p>
            The shard came back up. Your accounts did not. Everything staged is
            gone — the cash you already sold is yours.
          </p>
          <div class="big">+{view.pending_contacts} contacts</div>
          <p>Contacts are permanent. Spend them on being harder to catch.</p>
          <button type="button" onClick={() => exec('prestige')}>
            New identity
          </button>
        </div>
      </div>
    );
  }

  if (view.sweep_pending) {
    return (
      <div class="overlay">
        <div class="modal warn">
          <h3>Sweep imminent</h3>
          <p>
            Exposure is maxed. They are already looking. You get one last chance
            to sell.
          </p>
          <p>
            <b>{formatCompactNumber(view.gold)} gold</b> is staged and will be
            seized if you face it now.
          </p>
          <div class="modal-actions">
            <button
              type="button"
              class="money"
              onClick={() => exec('sell')}
              disabled={view.gold < 1}
            >
              Sell for {formatMoney(view.sell_proceeds)}
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
