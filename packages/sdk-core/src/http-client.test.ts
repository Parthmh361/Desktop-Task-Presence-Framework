import { describe, expect, it } from 'vitest';
import { parseTask } from './http-client';

describe('parseTask', () => {
  it('maps snake_case API fields to Task shape', () => {
    const task = parseTask({
      id: 'task-1',
      title: 'Buy milk',
      body: '2% organic',
      status: 'active',
      priority: 2,
      color: '#FFE066',
      remindAt: '2026-06-18T12:00:00.000Z',
      position: { x: 120, y: 340 },
      monitorId: 'monitor-1',
      sourceAppId: 'demo-app',
      sourceAppName: 'Demo App',
      metadata: { lane: 'inbox' },
      createdAt: '2026-06-18T10:00:00.000Z',
      updatedAt: '2026-06-18T11:00:00.000Z',
    });

    expect(task).toMatchObject({
      id: 'task-1',
      title: 'Buy milk',
      body: '2% organic',
      status: 'active',
      priority: 2,
      color: '#FFE066',
      monitorId: 'monitor-1',
      sourceAppId: 'demo-app',
      sourceAppName: 'Demo App',
      metadata: { lane: 'inbox' },
      position: { x: 120, y: 340 },
    });
    expect(task.remindAt).toEqual(new Date('2026-06-18T12:00:00.000Z'));
    expect(task.createdAt).toEqual(new Date('2026-06-18T10:00:00.000Z'));
    expect(task.updatedAt).toEqual(new Date('2026-06-18T11:00:00.000Z'));
  });

  it('defaults optional fields and coerces id/title to strings', () => {
    const task = parseTask({
      id: 42,
      title: 100,
      status: 'active',
    });

    expect(task).toEqual({
      id: '42',
      title: '100',
      body: undefined,
      status: 'active',
      priority: 0,
      color: undefined,
      remindAt: undefined,
      position: undefined,
      monitorId: undefined,
      sourceAppId: undefined,
      sourceAppName: undefined,
      metadata: undefined,
      createdAt: undefined,
      updatedAt: undefined,
    });
  });
});
