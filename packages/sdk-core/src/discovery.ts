import { DISCOVERY_TIMEOUT_MS, DTPFError, WELL_KNOWN_PORTS } from './types';

const DISCOVERY_HOSTS = ['localhost', '127.0.0.1'] as const;

export async function discoverAgent(timeout = DISCOVERY_TIMEOUT_MS): Promise<string> {
  // Prefer the same hostname the web app uses (localhost vs 127.0.0.1 matters in browsers)
  const hosts =
    typeof window !== 'undefined' && window.location.hostname
      ? [
          window.location.hostname,
          ...DISCOVERY_HOSTS.filter((h) => h !== window.location.hostname),
        ]
      : [...DISCOVERY_HOSTS];

  for (const host of hosts) {
    for (const port of WELL_KNOWN_PORTS) {
      try {
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), timeout);
        const res = await fetch(`http://${host}:${port}/health`, {
          signal: controller.signal,
        });
        clearTimeout(timer);
        if (res.ok) {
          return `http://${host}:${port}`;
        }
      } catch {
        // try next host/port
      }
    }
  }

  throw new DTPFError(
    'DTPF agent not running. Please install and start it.',
    'AGENT_NOT_FOUND',
  );
}
