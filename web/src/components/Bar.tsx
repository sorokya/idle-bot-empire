interface BarProps {
  percent: number;
  tone?: string;
}

export function Bar({ percent, tone = '' }: BarProps) {
  const width = Math.min(100, Math.max(0, percent));
  return (
    <div class={`bar ${tone}`}>
      <i style={{ width: `${width}%` }} />
    </div>
  );
}
