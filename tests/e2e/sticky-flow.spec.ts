import { test, expect } from '@playwright/test';

const agentReachable = async (baseURL: string | undefined): Promise<boolean> => {
  if (!baseURL) return false;
  try {
    const res = await fetch(`${baseURL}/health`, { signal: AbortSignal.timeout(2000) });
    return res.ok;
  } catch {
    return false;
  }
};

test.describe('DTPF agent sticky flow', () => {
  test('health endpoint responds when agent is running', async ({ baseURL }, testInfo) => {
    const reachable = await agentReachable(baseURL);
    test.skip(
      !reachable,
      'DTPF agent is not running — start it with pnpm agent:dev or set DTPF_AGENT_URL',
    );

    const response = await fetch(`${baseURL}/health`);
    expect(response.ok).toBe(true);

    const body = (await response.json()) as { version?: string; taskCount?: number };
    expect(body.version).toBeTruthy();
    expect(typeof body.taskCount).toBe('number');

    testInfo.annotations.push({
      type: 'note',
      description: 'Expand with sticky-note UI flows once agent startup is wired into CI.',
    });
  });
});
