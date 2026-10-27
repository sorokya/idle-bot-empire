import { useEffect, useRef } from 'preact/hooks';
import type { ViewModel } from '../types';

interface LogPaneProps {
  view: ViewModel;
}

export function LogPane({ view }: LogPaneProps) {
  const logRef = useRef<HTMLDivElement>(null);
  const atBottom = useRef(true);

  useEffect(() => {
    const element = logRef.current;
    if (!element || !atBottom.current) return;
    element.scrollTop = element.scrollHeight;
  }, [view.log.length]);

  return (
    <div
      class="log"
      ref={logRef}
      onScroll={(event) => {
        const element = event.currentTarget;
        atBottom.current =
          element.scrollHeight - element.scrollTop - element.clientHeight < 24;
      }}
    >
      {view.log.map((entry) => (
        <div key={entry.seq} class={`kind-${entry.kind.toLowerCase()}`}>
          <span class="t">{entry.clock}</span>
          <span class="m">{entry.text}</span>
        </div>
      ))}
    </div>
  );
}
