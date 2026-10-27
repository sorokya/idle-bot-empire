import { useCallback, useEffect, useRef, useState } from 'preact/hooks';
import type { ViewModel } from './types';
import init, { IbeGame } from './wasm/ibe_wasm.js';

const SAVE_KEY = 'idle-bot-empire/save/v1';
const SAVE_INTERVAL_MS = 5000;
const TICK_INTERVAL_MS = 250;

let suppressSave = false;

function persist(game: IbeGame) {
  if (suppressSave) return;
  try {
    localStorage.setItem(SAVE_KEY, game.save(Date.now() / 1000));
  } catch {
    // Storage quota or privacy mode: keep playing without persistence.
  }
}

function readView(game: IbeGame): ViewModel {
  // WASM boundary: the generated type is `any`, the Rust ViewModel is the contract.
  return game.view() as ViewModel;
}

async function boot(): Promise<{
  game: IbeGame;
  view: ViewModel;
  awaySeconds: number;
}> {
  await init();

  const rawSave = localStorage.getItem(SAVE_KEY);
  let game: IbeGame;
  if (rawSave) {
    try {
      game = IbeGame.loadSave(rawSave, Date.now() / 1000);
    } catch (error) {
      console.warn('save failed to load, starting fresh', error);
      game = new IbeGame(Date.now() % 1_000_000);
    }
  } else {
    game = new IbeGame(Date.now() % 1_000_000);
  }

  const view = readView(game);
  return { game, view, awaySeconds: view.offline_secs };
}

export function useGame() {
  const [view, setView] = useState<ViewModel | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [away, setAway] = useState(0);
  const gameRef = useRef<IbeGame | null>(null);

  useEffect(() => {
    let cancelled = false;
    let tickId = 0;
    let saveId = 0;

    void boot().then(({ game, view: firstView, awaySeconds }) => {
      if (cancelled) return;
      gameRef.current = game;
      setView(firstView);
      setAway(awaySeconds);

      let last = Date.now();
      tickId = window.setInterval(() => {
        const now = Date.now();
        game.tick(now - last);
        last = now;
        setView(readView(game));
      }, TICK_INTERVAL_MS);
      saveId = window.setInterval(() => persist(game), SAVE_INTERVAL_MS);
      window.addEventListener('pagehide', () => persist(game));

      if (import.meta.env.DEV) {
        window.ibe = {
          ff: (seconds) => {
            game.tick(seconds * 1000);
            const nextView = readView(game);
            setView(nextView);
            return nextView;
          },
          view: () => readView(game),
          reset: () => {
            suppressSave = true;
            localStorage.removeItem(SAVE_KEY);
            location.reload();
          },
        };
      }
    });

    return () => {
      cancelled = true;
      window.clearInterval(tickId);
      window.clearInterval(saveId);
    };
  }, []);

  const exec = useCallback((command: string) => {
    const game = gameRef.current;
    if (!game) return;
    const dispatchError = game.dispatch(command);
    setError(dispatchError ?? null);
    setView(readView(game));
  }, []);

  return { view, exec, error, away };
}
