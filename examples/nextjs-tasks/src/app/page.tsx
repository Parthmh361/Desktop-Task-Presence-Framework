'use client';

import { useState } from 'react';
import {
  useAgentStatus,
  useDTPFContext,
  useStickyTask,
} from '@dtpf/sdk-react';

export default function HomePage() {
  const status = useAgentStatus();
  const { isConnected, isConnecting, connectionError } = useDTPFContext();
  const { createTask, loading, error } = useStickyTask();
  const [toast, setToast] = useState<string | null>(null);

  const acceptTask = async () => {
    await createTask({
      title: 'Fix login bug',
      body: 'Investigate OAuth redirect loop on Safari',
      priority: 2,
      color: '#FFE066',
    });
    setToast('Sticky note created on your desktop!');
    setTimeout(() => setToast(null), 3000);
  };

  return (
    <div className="min-h-screen">
      {isConnecting && (
        <div className="bg-blue-50 border-b border-blue-200 px-4 py-2 text-sm text-blue-900">
          Connecting to DTPF Agent… Approve the dialog on your desktop if prompted.
        </div>
      )}
      {!isConnecting && !isConnected && (
        <div className="bg-amber-100 border-b border-amber-300 px-4 py-2 text-sm text-amber-900">
          DTPF Agent not connected.
          {connectionError ? ` ${connectionError}` : ' Install and start the desktop agent.'}
        </div>
      )}

      <header className="border-b bg-white px-6 py-4 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">Next.js Task Demo</h1>
          <p className="text-sm text-gray-500">Create a sticky note on your desktop</p>
        </div>
        <div className="flex items-center gap-2 text-sm">
          <span
            className={`w-2 h-2 rounded-full ${status ? 'bg-green-500' : 'bg-red-400'}`}
          />
          {status
            ? `Agent v${status.version} · ${status.taskCount} stickies`
            : 'Disconnected'}
        </div>
      </header>

      <main className="max-w-lg mx-auto p-6">
        <div className="bg-white rounded-lg border p-4 shadow-sm">
          <h2 className="font-medium mb-1">Fix login bug</h2>
          <p className="text-sm text-gray-600 mb-4">
            Investigate OAuth redirect loop on Safari
          </p>
          <button
            type="button"
            onClick={acceptTask}
            disabled={loading || !isConnected}
            className="px-3 py-1.5 rounded-md bg-blue-600 text-white text-sm hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Accept Task
          </button>
        </div>

        {error && <p className="mt-4 text-sm text-red-600">{error.message}</p>}
      </main>

      {toast && (
        <div className="fixed bottom-6 right-6 bg-gray-900 text-white px-4 py-2 rounded-lg shadow-lg text-sm">
          {toast}
        </div>
      )}
    </div>
  );
}
