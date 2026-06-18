import type { RegisterAppRequest, RegisterAppResponse } from '@dtpf/shared-types';
import { DTPFError } from './types';

export function getStoredToken(appId: string): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(`dtpf_token_${appId}`);
}

export function storeToken(appId: string, token: string): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(`dtpf_token_${appId}`, token);
}

export function clearStoredToken(appId: string): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.removeItem(`dtpf_token_${appId}`);
}

export async function registerApp(
  baseUrl: string,
  request: RegisterAppRequest,
  timeout: number,
): Promise<string> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);

  try {
    const res = await fetch(`${baseUrl}/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_id: request.appId,
        app_name: request.appName,
        origin: request.origin,
      }),
      signal: controller.signal,
    });

    if (res.status === 403) {
      throw new DTPFError('Authorization denied by user', 'AUTH_FAILED');
    }

    if (!res.ok) {
      throw new DTPFError(`Registration failed: ${res.statusText}`, 'AUTH_FAILED');
    }

    const data = (await res.json()) as RegisterAppResponse & { token: string; app_id: string };
    const token = data.token;
    storeToken(request.appId, token);
    return token;
  } catch (err) {
    if (err instanceof DTPFError) throw err;
    if (err instanceof DOMException && err.name === 'AbortError') {
      throw new DTPFError(
        'Registration timed out. Click Allow in the DTPF agent dialog on your desktop.',
        'NETWORK_ERROR',
        err,
      );
    }
    throw new DTPFError('Network error during registration', 'NETWORK_ERROR', err);
  } finally {
    clearTimeout(timer);
  }
}

export function getOrigin(): string {
  if (typeof window !== 'undefined' && window.location?.origin) {
    return window.location.origin;
  }
  return 'http://localhost:5173';
}
