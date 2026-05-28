import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import SessionRow from '../molecules/SessionRow';
import type { RsvpStatus, Session } from '../../types';
import { isPast, isLateJoinable, fmtTime } from '../../utils/date';
import { config } from '../../config';

const DAY_LIMIT = 2;
const LATER_LIMIT = 5;

function dayKey(d: Date) { return d.toISOString().slice(0, 10); }

interface Props {
  sessions: Session[];
  onRsvp: (id: string, status: RsvpStatus | null) => void;
}

export default function WeekCalendar({ sessions, onRsvp }: Props) {
  const { t } = useTranslation();
  const [showAllLater, setShowAllLater] = useState(false);

  const today = new Date(); today.setHours(0, 0, 0, 0);
  const weekEnd = new Date(today); weekEnd.setDate(weekEnd.getDate() + 7);
  const days = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(today); d.setDate(d.getDate() + i); return d;
  });

  const futureSessions = sessions.filter((s) => !isPast(s.scheduled_at) || isLateJoinable(s.scheduled_at));
  const byDay = new Map<string, Session[]>();
  const later: Session[] = [];

  for (const s of futureSessions) {
    const d = new Date(s.scheduled_at); d.setHours(0, 0, 0, 0);
    if (d >= weekEnd) { later.push(s); continue; }
    const k = dayKey(d);
    if (!byDay.has(k)) byDay.set(k, []);
    byDay.get(k)!.push(s);
  }

  const visibleLater = showAllLater ? later : later.slice(0, LATER_LIMIT);

  return (
    <div className="space-y-4">
      <div className="overflow-x-auto">
        <div className="grid grid-cols-7 gap-1.5 min-w-[560px]">
          {days.map((day) => {
            const k = dayKey(day);
            const isToday = k === dayKey(today);
            const daySessions = byDay.get(k) ?? [];
            const visible = daySessions.slice(0, DAY_LIMIT);
            const overflow = daySessions.length - DAY_LIMIT;

            return (
              <div key={k}>
                <div className={`text-center text-xs py-1.5 mb-1.5 rounded-lg ${
                  isToday ? 'bg-violet-600/20 text-violet-300 font-semibold' : 'text-zinc-500'
                }`}>
                  {day.toLocaleDateString(undefined, { weekday: 'short' })}
                  <br />
                  <span className="text-[10px]">
                    {day.toLocaleDateString(undefined, { day: '2-digit', month: '2-digit' })}
                  </span>
                </div>
                <div className="space-y-1.5">
                  {visible.map((s) => {
                    const thumbSrc = s.thumbnail_url
                      ? `${config.apiUrl}/api/v1/library/${encodeURIComponent(s.game)}/thumbnail`
                      : undefined;
                    return (
                      <Link
                        key={s.id}
                        to={`/sessions/${s.id}`}
                        className="block bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden hover:border-zinc-700 transition-colors"
                      >
                        {thumbSrc && (
                          <img
                            src={thumbSrc}
                            alt={s.game}
                            className="w-full h-10 object-cover"
                            onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
                          />
                        )}
                        <div className="p-1.5 text-xs space-y-0.5">
                          <p className="font-medium text-zinc-100 truncate leading-tight">{s.game}</p>
                          <p className="text-zinc-500">{fmtTime(s.scheduled_at)}</p>
                          <div className="flex items-center gap-1.5">
                            <span className="text-zinc-500">{s.participant_count}P</span>
                            {s.is_participant && (
                              <span className="text-emerald-400 text-[10px] font-medium">✓</span>
                            )}
                          </div>
                        </div>
                      </Link>
                    );
                  })}

                  {overflow > 0 && (
                    <Link
                      to={`/sessions/${daySessions[DAY_LIMIT].id}`}
                      className="block text-center text-[10px] text-zinc-600 hover:text-zinc-400 transition-colors py-0.5"
                    >
                      +{overflow} {t('week_calendar.show_more', { count: overflow }).replace('▼ ', '')}
                    </Link>
                  )}

                  {daySessions.length === 0 && (
                    <div className="h-8 rounded-lg border border-dashed border-zinc-800" />
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {later.length > 0 && (
        <div className="space-y-2">
          <p className="text-xs text-zinc-600 uppercase tracking-wider font-medium">{t('week_calendar.later_section')}</p>
          <ul className="divide-y divide-zinc-800 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            {visibleLater.map((s) => (
              <SessionRow key={s.id} session={s} onRsvp={onRsvp} />
            ))}
          </ul>
          {later.length > LATER_LIMIT && (
            <button
              onClick={() => setShowAllLater((v) => !v)}
              className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
            >
              {showAllLater
                ? t('week_calendar.show_less')
                : t('week_calendar.show_more', { count: later.length - LATER_LIMIT })}
            </button>
          )}
        </div>
      )}

      {futureSessions.length === 0 && (
        <p className="text-zinc-500 text-sm">{t('week_calendar.no_sessions')}</p>
      )}
    </div>
  );
}
