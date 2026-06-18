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
}

function getTaskId(): string {
  const params = new URLSearchParams(window.location.search);
  return params.get('taskId') ?? '';
}

const PRIORITY_COLORS = ['#94a3b8', '#3b82f6', '#f59e0b', '#ef4444'];

export default function App() {
  const [task, setTask] = useState<TaskData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const taskId = getTaskId();

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
      <div className="p-3 text-sm text-red-700 bg-white rounded-xl shadow-lg">
        {error}
      </div>
    );
  }

  if (!task) {
    return (
      <div className="p-3 text-sm text-gray-600 bg-yellow-100 rounded-xl shadow-lg animate-pulse">
        Loading...
      </div>
    );
  }

  const bg = task.color ?? '#FFE066';

  return (
    <div
      className="flex flex-col w-[280px] min-h-[200px] rounded-xl shadow-xl border border-black/10 overflow-hidden"
      style={{ backgroundColor: bg }}
    >
      <div
        className="h-6 cursor-grab active:cursor-grabbing bg-black/10 flex items-center justify-center"
        onMouseDown={onDragStart}
        onMouseUp={onDragEnd}
      >
        <div className="w-10 h-1 rounded bg-black/20" />
      </div>

      <div className="flex-1 p-3 flex flex-col gap-2">
        <div className="flex items-start gap-2">
          <span
            className="mt-1 w-2 h-2 rounded-full shrink-0"
            style={{ backgroundColor: PRIORITY_COLORS[task.priority] ?? PRIORITY_COLORS[0] }}
          />
          <h1 className="font-bold text-sm text-gray-900 leading-tight flex-1">{task.title}</h1>
        </div>

        {task.body && (
          <p className="text-xs text-gray-800 whitespace-pre-wrap flex-1">{task.body}</p>
        )}

        <div className="flex items-center justify-between mt-auto pt-2">
          <span className="text-[10px] text-gray-700/80 truncate max-w-[120px]">
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
