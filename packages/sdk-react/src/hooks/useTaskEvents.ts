import { useEffect, useRef } from 'react';
import type { TaskEvent } from '@dtpf/shared-types';
import { useDTPFContext } from '../context/DTPFProvider';

export function useTaskEvents(handler: (event: TaskEvent) => void): void {
  const { client } = useDTPFContext();
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    return client.subscribeTaskEvents((event) => handlerRef.current(event));
  }, [client]);
}
