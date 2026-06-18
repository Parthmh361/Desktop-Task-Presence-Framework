import type {
  AgentStatus,
  CreateTaskOptions,
  HealthResponse,
  Task,
  UpdateTaskOptions,
} from '@dtpf/shared-types';
import { DTPFError } from './types';

interface HttpClientOptions {
  baseUrl: string;
  appId: string;
  token: string;
  timeout: number;
}

function parseTask(raw: Record<string, unknown>): Task {
  return {
    id: String(raw.id),
    title: String(raw.title),
    body: raw.body ? String(raw.body) : undefined,
    status: raw.status as Task['status'],
    priority: (raw.priority ?? 0) as Task['priority'],
    color: raw.color ? String(raw.color) : undefined,
    remindAt: raw.remindAt ? new Date(String(raw.remindAt)) : undefined,
    position:
      raw.position && typeof raw.position === 'object'
        ? {
            x: Number((raw.position as { x: number }).x),
            y: Number((raw.position as { y: number }).y),
          }
        : undefined,
    monitorId: raw.monitorId ? String(raw.monitorId) : undefined,
    sourceAppId: raw.sourceAppId ? String(raw.sourceAppId) : undefined,
    sourceAppName: raw.sourceAppName ? String(raw.sourceAppName) : undefined,
    metadata: raw.metadata as Record<string, unknown> | undefined,
    createdAt: raw.createdAt ? new Date(String(raw.createdAt)) : undefined,
    updatedAt: raw.updatedAt ? new Date(String(raw.updatedAt)) : undefined,
  };
}

export class HttpClient {
  constructor(private options: HttpClientOptions) {}

  private headers(): HeadersInit {
    return {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${this.options.token}`,
      'X-DTPF-App-ID': this.options.appId,
      Origin: typeof window !== 'undefined' ? window.location.origin : '',
    };
  }

  private async request<T>(
    path: string,
    init: RequestInit = {},
  ): Promise<T> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.options.timeout);

    try {
      const res = await fetch(`${this.options.baseUrl}${path}`, {
        ...init,
        headers: { ...this.headers(), ...(init.headers ?? {}) },
        signal: controller.signal,
      });

      if (res.status === 401 || res.status === 403) {
        throw new DTPFError('Authentication failed', 'AUTH_FAILED');
      }
      if (res.status === 429) {
        throw new DTPFError('Rate limit exceeded', 'RATE_LIMITED');
      }
      if (res.status === 404) {
        throw new DTPFError('Task not found', 'TASK_NOT_FOUND');
      }
      if (!res.ok) {
        throw new DTPFError(`Request failed: ${res.statusText}`, 'NETWORK_ERROR');
      }

      if (res.status === 204) {
        return undefined as T;
      }

      return (await res.json()) as T;
    } catch (err) {
      if (err instanceof DTPFError) throw err;
      throw new DTPFError('Network error', 'NETWORK_ERROR', err);
    } finally {
      clearTimeout(timer);
    }
  }

  async getHealth(): Promise<HealthResponse & { platform?: string }> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.options.timeout);
    try {
      const res = await fetch(`${this.options.baseUrl}/health`, {
        signal: controller.signal,
      });
      if (!res.ok) throw new DTPFError('Agent health check failed', 'NETWORK_ERROR');
      return (await res.json()) as HealthResponse & { platform?: string };
    } finally {
      clearTimeout(timer);
    }
  }

  async listTasks(): Promise<Task[]> {
    const data = await this.request<Record<string, unknown>[]>('/tasks');
    return data.map(parseTask);
  }

  async getTask(id: string): Promise<Task | null> {
    try {
      const data = await this.request<Record<string, unknown>>(`/tasks/${id}`);
      return parseTask(data);
    } catch (err) {
      if (err instanceof DTPFError && err.code === 'TASK_NOT_FOUND') return null;
      throw err;
    }
  }

  async createTask(options: CreateTaskOptions): Promise<Task> {
    const body = {
      title: options.title,
      body: options.body,
      priority: options.priority,
      color: options.color,
      remind_at: options.remindAt?.getTime(),
      position: options.position,
      monitor_id: options.monitorId,
      metadata: options.metadata,
    };
    const data = await this.request<Record<string, unknown>>('/tasks', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    return parseTask(data);
  }

  async updateTask(id: string, options: UpdateTaskOptions): Promise<Task> {
    const body = {
      title: options.title,
      body: options.body,
      priority: options.priority,
      color: options.color,
      remind_at: options.remindAt?.getTime(),
      metadata: options.metadata,
    };
    const data = await this.request<Record<string, unknown>>(`/tasks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(body),
    });
    return parseTask(data);
  }

  async deleteTask(id: string): Promise<void> {
    await this.request<void>(`/tasks/${id}`, { method: 'DELETE' });
  }

  async completeTask(id: string): Promise<void> {
    await this.request<Record<string, unknown>>(`/tasks/${id}/complete`, {
      method: 'POST',
    });
  }

  async snoozeTask(id: string, until: Date): Promise<Task> {
    const data = await this.request<Record<string, unknown>>(`/tasks/${id}/snooze`, {
      method: 'POST',
      body: JSON.stringify({ until: until.getTime() }),
    });
    return parseTask(data);
  }

  async getAgentStatus(): Promise<AgentStatus> {
    const health = await this.getHealth();
    return {
      connected: true,
      version: health.version,
      platform: (health.platform as AgentStatus['platform']) ?? 'linux',
      taskCount: health.taskCount,
    };
  }
}

export { parseTask };
