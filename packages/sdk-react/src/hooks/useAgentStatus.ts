import { useEffect, useState } from 'react';
import type { AgentStatus, TaskEvent } from '@dtpf/shared-types';
import { useDTPFContext } from '../context/DTPFProvider';

export function useAgentStatus(): AgentStatus | null {
  const { client, status: initialStatus, isConnected } = useDTPFContext();
  const [status, setStatus] = useState<AgentStatus | null>(initialStatus);

  useEffect(() => {
    setStatus(initialStatus);
  }, [initialStatus]);

  useEffect(() => {
    if (!isConnected) {
      return;
    }

    const refresh = async () => {
      try {
        const next = await client.getAgentStatus();
        setStatus(next);
      } catch {
        setStatus(null);
      }
    };

    refresh();

    const interval = setInterval(refresh, 30_000);

    const unsub = client.subscribeTaskEvents((event: TaskEvent) => {
      if (
        event.type === 'task:created' ||
        event.type === 'task:updated' ||
        event.type === 'task:completed' ||
        event.type === 'task:dismissed'
      ) {
        refresh();
      }
    });

    return () => {
      clearInterval(interval);
      unsub();
    };
  }, [client, isConnected]);

  return isConnected ? status : null;
}
