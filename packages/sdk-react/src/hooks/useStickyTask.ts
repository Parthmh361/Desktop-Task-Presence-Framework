import { useCallback, useState } from 'react';
import type { CreateTaskOptions, Task, UpdateTaskOptions } from '@dtpf/shared-types';
import { useDTPFContext } from '../context/DTPFProvider';

export function useStickyTask() {
  const { client } = useDTPFContext();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const run = useCallback(
    async <T,>(fn: () => Promise<T>): Promise<T> => {
      setLoading(true);
      setError(null);
      try {
        return await fn();
      } catch (err) {
        const e = err instanceof Error ? err : new Error(String(err));
        setError(e);
        throw e;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const createTask = useCallback(
    (opts: CreateTaskOptions) => run(() => client.createStickyTask(opts)),
    [client, run],
  );

  const updateTask = useCallback(
    (id: string, opts: UpdateTaskOptions) =>
      run(() => client.updateStickyTask(id, opts)),
    [client, run],
  );

  const deleteTask = useCallback(
    (id: string) => run(() => client.deleteStickyTask(id)),
    [client, run],
  );

  const completeTask = useCallback(
    (id: string) => run(() => client.completeTask(id)),
    [client, run],
  );

  return { createTask, updateTask, deleteTask, completeTask, loading, error };
}

export type { Task, CreateTaskOptions, UpdateTaskOptions };
