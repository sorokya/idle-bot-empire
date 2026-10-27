const COMPACT_UNITS = ['', 'K', 'M', 'B', 'T', 'Qa', 'Qi', 'Sx'] as const;

export function formatCompactNumber(input: number): string {
  if (!Number.isFinite(input)) return '0';
  if (input < 1000) return Math.floor(input).toString();

  let value = input;
  let unitIndex = 0;
  while (value >= 1000 && unitIndex < COMPACT_UNITS.length - 1) {
    value /= 1000;
    unitIndex += 1;
  }

  const decimals = value < 10 ? 2 : value < 100 ? 1 : 0;
  return `${value.toFixed(decimals)}${COMPACT_UNITS[unitIndex] ?? ''}`;
}

export function formatMoney(value: number): string {
  return `$${value.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

export function formatDuration(seconds: number): string {
  const totalSeconds = Math.max(0, Math.floor(seconds));
  const days = Math.floor(totalSeconds / 86_400);
  const hours = Math.floor((totalSeconds % 86_400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);

  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}
