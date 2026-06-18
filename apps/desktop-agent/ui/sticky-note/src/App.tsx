import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface TaskData {
  id: string;
  title: string;
  body?: string;
  status: string;
  priority: number;
  color?: string;
  sourceAppName?: string;
  remindAt?: string | null;
}

function getTaskId(): string {
  const params = new URLSearchParams(window.location.search);
  return params.get('taskId') ?? '';
}

function parseRemindAt(value: unknown): number | null {
  if (value == null) return null;
  if (typeof value === 'number') return value;
  if (typeof value === 'string') {
    const ms = Date.parse(value);
    return Number.isNaN(ms) ? null : ms;
  }
  return null;
}

function formatCountdown(msRemaining: number): string {
  if (msRemaining <= 0) return 'Due now';
  const totalSec = Math.floor(msRemaining / 1000);
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

const PRIORITY_COLORS = ['#94a3b8', '#3b82f6', '#f59e0b', '#ef4444'];

function useCountdown(remindAtMs: number | null): string | null {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    if (remindAtMs == null) {
      setLabel(null);
      return;
    }

    const tick = () => {
      setLabel(formatCountdown(remindAtMs - Date.now()));
    };
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  }, [remindAtMs]);

  return label;
}

export default function App() {
  const [task, setTask] = useState<TaskData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dark, setDark] = useState(false);
  const taskId = getTaskId();

  const remindAtMs = task ? parseRemindAt(task.remindAt) : null;
  const countdown = useCountdown(remindAtMs);

  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    setDark(mq.matches);
    const handler = (e: MediaQueryListEvent) => setDark(e.matches);
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, []);

  useEffect(() => {
    if (!taskId) {
      setError('No task ID');
      return;
    }

    invoke<TaskData>('get_task', { taskId })
      .then(setTask)
      .catch((e) => setError(String(e)));

    const unlisten = listen<TaskData>('task:updated', (event) => {
      if (event.payload.id === taskId) {
        setTask(event.payload);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [taskId]);

  const onDragStart = useCallback(async () => {
    await getCurrentWindow().startDragging();
  }, []);

  const onDragEnd = useCallback(async () => {
    const pos = await getCurrentWindow().outerPosition();
    await invoke('save_window_position', {
      taskId,
      x: pos.x,
      y: pos.y,
    });
  }, [taskId]);

  const complete = async () => {
    await invoke('complete_task_from_ui', { taskId });
    await getCurrentWindow().close();
  };

  const dismiss = async () => {
    await invoke('dismiss_task_from_ui', { taskId });
    await getCurrentWindow().close();
  };

  if (error) {
    return (
      <div className="p-3 text-sm text-red-700 dark:text-red-300 bg-white dark:bg-gray-900 rounded-xl shadow-lg">
        {error}
      </div>
    );
  }

  if (!task) {
    return (
      <div className="p-3 text-sm text-gray-600 dark:text-gray-300 bg-yellow-100 dark:bg-yellow-900/40 rounded-xl shadow-lg animate-pulse">
        Loading...
      </div>
    );
  }

  const bg = task.color ?? '#FFE066';
  const titleClass = dark ? 'text-gray-100' : 'text-gray-900';
  const bodyClass = dark ? 'text-gray-200' : 'text-gray-800';
  const metaClass = dark ? 'text-gray-300/90' : 'text-gray-700/80';

  return (
    <div
      className="flex flex-col w-[280px] min-h-[200px] rounded-xl shadow-xl border border-black/10 dark:border-white/10 overflow-hidden"
      style={{ backgroundColor: bg }}
    >
      <div
        className="h-6 cursor-grab active:cursor-grabbing bg-black/10 dark:bg-white/10 flex items-center justify-center gap-2"
        onMouseDown={onDragStart}
        onMouseUp={onDragEnd}
      >
        <div className="w-10 h-1 rounded bg-black/20 dark:bg-white/30" />
        {task.status === 'snoozed' && (
          <span className="text-[9px] font-semibold uppercase tracking-wide px-1.5 py-0.5 rounded bg-blue-600/80 text-white">
            Snoozed
          </span>
        )}
      </div>

      <div className="flex-1 p-3 flex flex-col gap-2">
        <div className="flex items-start gap-2">
          <span
            className="mt-1 w-2 h-2 rounded-full shrink-0"
            style={{ backgroundColor: PRIORITY_COLORS[task.priority] ?? PRIORITY_COLORS[0] }}
          />
          <h1 className={`font-bold text-sm leading-tight flex-1 ${titleClass}`}>{task.title}</h1>
        </div>

        {task.body && (
          <p className={`text-xs whitespace-pre-wrap flex-1 ${bodyClass}`}>{task.body}</p>
        )}

        {countdown && (
          <p className={`text-[10px] font-medium ${dark ? 'text-amber-200' : 'text-amber-900'}`}>
            Reminder in {countdown}
          </p>
        )}

        <div className="flex items-center justify-between mt-auto pt-2">
          <span className={`text-[10px] truncate max-w-[120px] ${metaClass}`}>
            {task.sourceAppName ?? 'DTPF'}
          </span>
          <div className="flex gap-1">
            <button
              onClick={complete}
              className="w-7 h-7 rounded-md bg-green-600/90 hover:bg-green-700 text-white text-sm"
              title="Complete"
            >
              ✓
            </button>
            <button
              onClick={dismiss}
              className="w-7 h-7 rounded-md bg-red-500/90 hover:bg-red-600 text-white text-sm"
              title="Dismiss"
            >
              ✕
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
