import type { DTPFErrorCode } from '@dtpf/shared-types';

export class DTPFError extends Error {
  constructor(
    message: string,
    public readonly code: DTPFErrorCode,
    public readonly cause?: unknown,
  ) {
    super(message);
    this.name = 'DTPFError';
  }
}

export type Unsubscribe = () => void;

export const WELL_KNOWN_PORTS = [7842, 7843, 7844] as const;
export const DISCOVERY_TIMEOUT_MS = 3000;
export const REGISTRATION_TIMEOUT_MS = 120_000;
