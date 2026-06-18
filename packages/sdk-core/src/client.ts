import type {
  AgentStatus,
  CreateTaskOptions,
  DTPFConfig,
  Task,
  TaskEvent,
  UpdateTaskOptions,
} from '@dtpf/shared-types';
import {
  clearStoredToken,
  getOrigin,
  getStoredToken,
  registerApp,
  storeToken,
} from './auth';
import { discoverAgent } from './discovery';
import { HttpClient } from './http-client';
import { DTPFError, REGISTRATION_TIMEOUT_MS, type Unsubscribe } from './types';
import { WsClient } from './ws-client';

export class DTPFClient {
  private baseUrl: string | null = null;
  private token: string | null = null;
  private http: HttpClient | null = null;
  private ws: WsClient | null = null;
  private eventHandlers = new Set<(event: TaskEvent) => void>();
  private connected = false;
  private readonly timeout: number;

  constructor(private readonly config: DTPFConfig) {
    this.timeout = config.timeout ?? 5000;
    if (config.token) {
      this.token = config.token;
      storeToken(config.appId, config.token);
    }
  }

  async connect(): Promise<void> {
    this.baseUrl = await discoverAgent(Math.min(this.timeout, 3000));

    let token = this.token ?? getStoredToken(this.config.appId);
    if (!token) {
      token = await this.register();
    }

    this.http = this.createHttpClient(token);

    // Validate stored token; re-register if stale
    try {
      await this.http.listTasks();
    } catch (err) {
      if (err instanceof DTPFError && err.code === 'AUTH_FAILED') {
        clearStoredToken(this.config.appId);
        token = await this.register();
        this.http = this.createHttpClient(token);
      } else {
        throw err;
      }
    }

    this.token = token;
    await this.http.getHealth();
    this.connected = true;
    this.emit({ type: 'agent:connected' });

    this.ws = new WsClient({
      baseUrl: this.baseUrl,
      appId: this.config.appId,
      token,
      onEvent: (event) => this.emit(event),
    });
    this.ws.connect();
  }

  private createHttpClient(token: string): HttpClient {
    return new HttpClient({
      baseUrl: this.baseUrl!,
      appId: this.config.appId,
      token,
      timeout: this.timeout,
    });
  }

  private register(): Promise<string> {
    return registerApp(
      this.baseUrl!,
      {
        appId: this.config.appId,
        appName: this.config.appName,
        origin: getOrigin(),
      },
      REGISTRATION_TIMEOUT_MS,
    );
  }

  async disconnect(): Promise<void> {
    this.ws?.disconnect();
    this.ws = null;
    this.http = null;
    this.connected = false;
    this.emit({ type: 'agent:disconnected' });
  }

  async getAgentStatus(): Promise<AgentStatus> {
    this.ensureConnected();
    return this.http!.getAgentStatus();
  }

  async createStickyTask(options: CreateTaskOptions): Promise<Task> {
    this.ensureConnected();
    return this.http!.createTask(options);
  }

  async updateStickyTask(taskId: string, options: UpdateTaskOptions): Promise<Task> {
    this.ensureConnected();
    return this.http!.updateTask(taskId, options);
  }

  async deleteStickyTask(taskId: string): Promise<void> {
    this.ensureConnected();
    return this.http!.deleteTask(taskId);
  }

  async completeTask(taskId: string): Promise<void> {
    this.ensureConnected();
    return this.http!.completeTask(taskId);
  }

  async snoozeTask(taskId: string, until: Date): Promise<Task> {
    this.ensureConnected();
    return this.http!.snoozeTask(taskId, until);
  }

  async getTask(taskId: string): Promise<Task | null> {
    this.ensureConnected();
    return this.http!.getTask(taskId);
  }

  async listTasks(): Promise<Task[]> {
    this.ensureConnected();
    return this.http!.listTasks();
  }

  async showReminder(_taskId: string, _message?: string): Promise<void> {
    this.ensureConnected();
  }

  subscribeTaskEvents(handler: (event: TaskEvent) => void): Unsubscribe {
    this.eventHandlers.add(handler);
    return () => this.eventHandlers.delete(handler);
  }

  unsubscribeAll(): void {
    this.eventHandlers.clear();
  }

  get isConnected(): boolean {
    return this.connected;
  }

  private ensureConnected(): void {
    if (!this.http) {
      throw new DTPFError('Not connected. Call connect() first.', 'AGENT_NOT_FOUND');
    }
  }

  private emit(event: TaskEvent): void {
    for (const handler of this.eventHandlers) {
      handler(event);
    }
  }
}

export { DTPFError } from './types';
export type { Unsubscribe } from './types';
