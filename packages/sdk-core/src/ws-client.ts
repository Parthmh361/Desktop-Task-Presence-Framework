import type { TaskEvent } from '@dtpf/shared-types';
import { parseTask } from './http-client';
import type { Unsubscribe } from './types';

interface WsClientOptions {
  baseUrl: string;
  appId: string;
  token: string;
  onEvent: (event: TaskEvent) => void;
}

export class WsClient {
  private ws: WebSocket | null = null;
  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private closed = false;

  constructor(private options: WsClientOptions) {}

  connect(): void {
    this.closed = false;
    const params = new URLSearchParams({
      token: this.options.token,
      app_id: this.options.appId,
    });
    const wsUrl =
      this.options.baseUrl.replace(/^http/, 'ws') + '/ws?' + params.toString();
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      this.reconnectAttempt = 0;
    };

    this.ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(String(msg.data)) as Record<string, unknown>;
        const event = this.parseEvent(data);
        if (event) this.options.onEvent(event);
      } catch {
        // ignore malformed events
      }
    };

    this.ws.onclose = () => {
      if (!this.closed) this.scheduleReconnect();
    };

    this.ws.onerror = () => {
      this.ws?.close();
    };
  }

  private parseEvent(data: Record<string, unknown>): TaskEvent | null {
    switch (data.type) {
      case 'task:created':
      case 'task:updated':
        return {
          type: data.type,
          task: parseTask(data.task as Record<string, unknown>),
        };
      case 'task:completed':
      case 'task:dismissed':
        return {
          type: data.type,
          taskId: String(data.taskId),
          task: data.task
            ? parseTask(data.task as Record<string, unknown>)
            : undefined,
        };
      case 'task:reminder':
        return {
          type: 'task:reminder',
          task: parseTask(data.task as Record<string, unknown>),
        };
      default:
        return null;
    }
  }

  private scheduleReconnect(): void {
    const delay = Math.min(1000 * 2 ** this.reconnectAttempt, 30000);
    this.reconnectAttempt += 1;
    this.reconnectTimer = setTimeout(() => this.connect(), delay);
  }

  disconnect(): void {
    this.closed = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.ws?.close();
    this.ws = null;
  }
}

export function createWsClient(options: WsClientOptions): WsClient {
  return new WsClient(options);
}

export type { Unsubscribe };
