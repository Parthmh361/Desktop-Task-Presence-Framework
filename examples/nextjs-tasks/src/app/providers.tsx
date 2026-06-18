'use client';

import { DTPFProvider } from '@dtpf/sdk-react';
import type { ReactNode } from 'react';

const DTPF_CONFIG = {
  appId: 'com.acme.tasks.nextjs',
  appName: 'Next.js Task Demo',
} as const;

export function Providers({ children }: { children: ReactNode }) {
  return <DTPFProvider config={DTPF_CONFIG}>{children}</DTPFProvider>;
}
