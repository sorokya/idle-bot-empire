import { useEffect, useState } from 'preact/hooks';
import { formatCompactNumber, formatDuration, formatMoney } from '../format';
import { useGame } from '../useGame';
import { AccountList } from './AccountList';
import { Commands } from './Commands';
import { LogPane } from './LogPane';
import { ManualPhase } from './ManualPhase';
import { Modals } from './Modals';
import { Stats } from './Stats';
import { TitleBar } from './TitleBar';

export function App() {
  const { view, exec, error, away } = useGame();
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  useEffect(() => {
    if (error) setToastMessage(error);
  }, [error]);

  useEffect(() => {
    if (!toastMessage) return;
    const timeoutId = window.setTimeout(() => setToastMessage(null), 2600);
    return () => window.clearTimeout(timeoutId);
  }, [toastMessage]);

  useEffect(() => {
    if (!view || away < 60) return;
    setToastMessage(
      `While you were away ${formatDuration(away)}: ${formatCompactNumber(view.gold)} gold, ${formatMoney(view.cash)} on hand.`,
    );
  }, [view, away]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey || event.altKey || !view) return;

      const commandMap: Record<string, string> = view.started
        ? {
            '1': 'buy_account',
            '2': 'buy_slot',
            '3': 'sell',
            '4': 'upgrade:script',
            '5': 'upgrade:proxy',
            '6': 'upgrade:humanizer',
          }
        : { '1': 'click', '2': 'sell', '3': 'buy_account' };

      const extraMap: Record<string, string> = {
        p: 'prestige',
        f: 'face_sweep',
        '[': `aggression:${Math.max(0.2, Math.round((view.aggression - 0.1) * 10) / 10)}`,
        ']': `aggression:${Math.min(2, Math.round((view.aggression + 0.1) * 10) / 10)}`,
      };

      const key = event.key.toLowerCase();
      const command = commandMap[key] ?? extraMap[key];
      if (command) {
        event.preventDefault();
        exec(command);
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
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
        <TitleBar view={view} />
        <Stats view={view} />
        {view.started ? (
          <Commands view={view} exec={exec} />
        ) : (
          <ManualPhase view={view} exec={exec} />
        )}
      </div>

      <div class="frame">
        <AccountList view={view} exec={exec} />
        <LogPane view={view} />
      </div>

      <div class="foot">
        1–6 act · [ ] greed · p new identity · {view.banned_total} banned
        all-time
      </div>

      <Modals view={view} exec={exec} />
      {toastMessage && <div class="toast show">{toastMessage}</div>}
    </>
  );
}
