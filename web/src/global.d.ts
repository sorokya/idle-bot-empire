import type { ViewModel } from './types';

declare global {
  interface Window {
    ibe?: {
      ff: (seconds: number) => ViewModel;
      view: () => ViewModel;
      reset: () => void;
    };
  }
}
