import { useCallback, useEffect, useState } from 'react';
import type { Task, TaskEvent } from '@dtpf/shared-types';
import { useDTPFContext } from '../context/DTPFProvider';

function applyTaskEvent(tasks: Task[], event: TaskEvent): Task[] {
  switch (event.type) {
    case 'task:created':
      return [...tasks.filter((t) => t.id !== event.task.id), event.task];
    case 'task:updated':
      return tasks.some((t) => t.id === event.task.id)
        ? tasks.map((t) => (t.id === event.task.id ? event.task : t))
        : [...tasks, event.task];
    case 'task:completed':
    case 'task:dismissed':
      return tasks.filter((t) => t.id !== event.taskId);
    default:
      return tasks;
  }
}

export function useTasks() {
  const { client, isConnected } = useDTPFContext();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!isConnected) {
      setTasks([]);
      return;
    }
    const next = await client.listTasks();
    setTasks(next);
  }, [client, isConnected]);

  useEffect(() => {
    if (!isConnected) {
      setTasks([]);
      setLoading(false);
      return;
    }

    let mounted = true;
    setLoading(true);
    client
      .listTasks()
      .then((next) => {
        if (mounted) setTasks(next);
      })
      .finally(() => {
        if (mounted) setLoading(false);
      });

    const unsub = client.subscribeTaskEvents((event) => {
      if (
        event.type === 'task:created' ||
        event.type === 'task:updated' ||
        event.type === 'task:completed' ||
        event.type === 'task:dismissed'
      ) {
        setTasks((prev) => applyTaskEvent(prev, event));
      }
    });

    return () => {
      mounted = false;
      unsub();
    };
  }, [client, isConnected]);

  const getTaskById = useCallback(
    (taskId: string) => tasks.find((t) => t.id === taskId) ?? null,
    [tasks],
  );

  const getTaskByMetadata = useCallback(
    (key: string, value: unknown) =>
      tasks.find((t) => t.metadata?.[key] === value) ?? null,
    [tasks],
  );

  return { tasks, loading, refresh, getTaskById, getTaskByMetadata };
}
