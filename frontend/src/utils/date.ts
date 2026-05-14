export function isPast(utcStr: string): boolean {
  return new Date(utcStr) < new Date();
}

/** True if the session started in the last 2 hours — still joinable. */
export function isLateJoinable(utcStr: string): boolean {
  const start = new Date(utcStr);
  const now = new Date();
  return start <= now && now.getTime() - start.getTime() <= 2 * 60 * 60 * 1000;
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
