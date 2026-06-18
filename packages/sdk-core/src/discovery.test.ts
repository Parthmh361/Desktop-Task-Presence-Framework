import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { discoverAgent } from './discovery';
import { WELL_KNOWN_PORTS } from './types';

describe('discoverAgent', () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    vi.stubGlobal('fetch', fetchMock);
    fetchMock.mockReset();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('returns base URL when health check succeeds on first port', async () => {
    fetchMock.mockResolvedValueOnce({
      ok: true,
    } as Response);

    const url = await discoverAgent(1000);

    expect(url).toBe(`http://localhost:${WELL_KNOWN_PORTS[0]}`);
    expect(fetchMock).toHaveBeenCalledWith(
      `http://localhost:${WELL_KNOWN_PORTS[0]}/health`,
      expect.objectContaining({ signal: expect.any(AbortSignal) }),
    );
  });

  it('probes fallback ports when earlier ports fail', async () => {
    fetchMock
      .mockRejectedValueOnce(new Error('connection refused'))
      .mockResolvedValueOnce({ ok: false } as Response)
      .mockResolvedValueOnce({ ok: true } as Response);

    const url = await discoverAgent(1000);

    expect(url).toBe(`http://localhost:${WELL_KNOWN_PORTS[2]}`);
    expect(fetchMock).toHaveBeenCalledTimes(3);
  });

  it('throws AGENT_NOT_FOUND when all host/port probes fail', async () => {
    fetchMock.mockRejectedValue(new Error('connection refused'));

    await expect(discoverAgent(100)).rejects.toMatchObject({
      code: 'AGENT_NOT_FOUND',
    });
  });
});
