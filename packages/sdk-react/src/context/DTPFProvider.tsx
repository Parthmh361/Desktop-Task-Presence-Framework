import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import { DTPFClient } from '@dtpf/sdk-core';
import type { AgentStatus, DTPFConfig } from '@dtpf/shared-types';

interface DTPFContextValue {
  client: DTPFClient;
  status: AgentStatus | null;
  isConnected: boolean;
  isConnecting: boolean;
  connectionError: string | null;
}

const DTPFContext = createContext<DTPFContextValue | null>(null);

export function DTPFProvider({
  config,
  children,
}: {
  config: DTPFConfig;
  children: ReactNode;
}) {
  const client = useMemo(
    () => new DTPFClient(config),
    [config.appId, config.appName, config.token, config.timeout],
  );
  const [status, setStatus] = useState<AgentStatus | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(true);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const connectAttempt = useRef(0);

  useEffect(() => {
    const attempt = ++connectAttempt.current;
    let mounted = true;

    setIsConnecting(true);
    setConnectionError(null);

    client
      .connect()
      .then(async () => {
        if (!mounted || attempt !== connectAttempt.current) return;
        setIsConnected(true);
        setIsConnecting(false);
        const agentStatus = await client.getAgentStatus();
        if (mounted && attempt === connectAttempt.current) setStatus(agentStatus);
      })
      .catch((err) => {
        if (!mounted || attempt !== connectAttempt.current) return;
        setIsConnected(false);
        setIsConnecting(false);
        setStatus(null);
        setConnectionError(err instanceof Error ? err.message : String(err));
      });

    const unsub = client.subscribeTaskEvents((event) => {
      if (event.type === 'agent:connected') setIsConnected(true);
      if (event.type === 'agent:disconnected') setIsConnected(false);
    });

    return () => {
      mounted = false;
      unsub();
      client.disconnect();
    };
  }, [client]);

  const value = useMemo(
    () => ({ client, status, isConnected, isConnecting, connectionError }),
    [client, status, isConnected, isConnecting, connectionError],
  );

  return <DTPFContext.Provider value={value}>{children}</DTPFContext.Provider>;
}

export function useDTPFContext(): DTPFContextValue {
  const ctx = useContext(DTPFContext);
  if (!ctx) {
    throw new Error('useDTPFContext must be used within DTPFProvider');
  }
  return ctx;
}

export { DTPFContext };
