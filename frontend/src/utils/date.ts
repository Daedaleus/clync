export function isPast(utcStr: string): boolean {
  return new Date(utcStr) < new Date();
}

export function fmtDateTime(utcStr: string): string {
  return new Date(utcStr).toLocaleString(undefined, {
    weekday: 'short', day: '2-digit', month: '2-digit',
    hour: '2-digit', minute: '2-digit',
  });
}

export function fmtTime(utcStr: string): string {
  return new Date(utcStr).toLocaleTimeString(undefined, {
    hour: '2-digit', minute: '2-digit',
  });
}
