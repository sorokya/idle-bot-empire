import { formatDuration } from '../format';
import type { ViewModel } from '../types';

interface TitleBarProps {
  view: ViewModel;
}

export function TitleBar({ view }: TitleBarProps) {
  return (
    <div class="titlebar">
      <span>
        <b>Idle Bot Empire</b> · shard 1
      </span>
      <span>
        day {view.day} · {view.clock} · run{' '}
        {formatDuration(view.run_hrs * 3600)}
      </span>
    </div>
  );
}
