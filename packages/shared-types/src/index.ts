export type TaskStatus = 'active' | 'completed' | 'dismissed' | 'snoozed';
export type TaskPriority = 0 | 1 | 2 | 3;

export interface Task {
  id: string;
  title: string;
  body?: string;
  status: TaskStatus;
  priority: TaskPriority;
  color?: string;
  remindAt?: Date;
  position?: { x: number; y: number };
  monitorId?: string;
  sourceAppId?: string;
  sourceAppName?: string;
  metadata?: Record<string, unknown>;
  createdAt?: Date;
  updatedAt?: Date;
}

export interface CreateTaskOptions {
  title: string;
  body?: string;
  priority?: TaskPriority;
  color?: string;
  remindAt?: Date;
  position?: { x: number; y: number };
  monitorId?: string;
  metadata?: Record<string, unknown>;
}

export interface UpdateTaskOptions {
  title?: string;
  body?: string;
  priority?: TaskPriority;
  color?: string;
  remindAt?: Date;
  metadata?: Record<string, unknown>;
}

export type TaskEvent =
  | { type: 'task:created'; task: Task }
  | { type: 'task:updated'; task: Task }
  | { type: 'task:completed'; taskId: string; task?: Task }
  | { type: 'task:dismissed'; taskId: string; task?: Task }
  | { type: 'task:reminder'; task: Task }
  | { type: 'agent:connected' }
  | { type: 'agent:disconnected' };

export interface AgentStatus {
  connected: boolean;
  version: string;
  platform: 'windows' | 'macos' | 'linux';
  taskCount: number;
}

export interface DTPFConfig {
  appId: string;
  appName: string;
  token?: string;
  cloudSyncUrl?: string;
  timeout?: number;
}

export type DTPFErrorCode =
  | 'AGENT_NOT_FOUND'
  | 'AUTH_FAILED'
  | 'RATE_LIMITED'
  | 'TASK_NOT_FOUND'
  | 'NETWORK_ERROR';

export interface RegisterAppRequest {
  appId: string;
  appName: string;
  origin: string;
}

export interface RegisterAppResponse {
  token: string;
  appId: string;
}

export interface HealthResponse {
  version: string;
  status: string;
  taskCount: number;
}
