import { useCallback, useState } from 'react';
import {
  DTPFProvider,
  useAgentStatus,
  useDTPFContext,
  useStickyTask,
  useTaskEvents,
  useTasks,
} from '@dtpf/sdk-react';

const DTPF_CONFIG = {
  appId: 'com.acme.tasks.demo',
  appName: 'Acme Task Manager',
} as const;

const SAMPLE_TASKS = [
  { id: '1', title: 'Fix login bug', body: 'Investigate OAuth redirect loop on Safari', priority: 2 as const },
  { id: '2', title: 'Deploy to staging', body: 'Run migration script before deploy', priority: 1 as const },
  { id: '3', title: 'Review PR #42', body: 'Focus on auth middleware changes', priority: 0 as const },
];

type SampleStatus = 'pending' | 'on-desktop' | 'completed' | 'dismissed';

const STATUS_LABELS: Record<SampleStatus, string> = {
  pending: 'Not sent',
  'on-desktop': 'On desktop',
  completed: 'Completed',
  dismissed: 'Dismissed',
};

const STATUS_COLORS: Record<SampleStatus, string> = {
  pending: 'bg-gray-100 text-gray-700',
  'on-desktop': 'bg-yellow-100 text-yellow-900',
  completed: 'bg-green-100 text-green-800',
  dismissed: 'bg-red-100 text-red-800',
};

function Dashboard() {
  const status = useAgentStatus();
  const { isConnected, isConnecting, connectionError } = useDTPFContext();
  const { tasks, getTaskByMetadata } = useTasks();
  const { createTask, completeTask, deleteTask, loading, error } = useStickyTask();
  const [toast, setToast] = useState<string | null>(null);
  const [terminalStatus, setTerminalStatus] = useState<Record<string, SampleStatus>>({});

  const showToast = useCallback((message: string) => {
    setToast(message);
    setTimeout(() => setToast(null), 3000);
  }, []);

  useTaskEvents((event) => {
    if (event.type === 'task:completed' || event.type === 'task:dismissed') {
      const sampleId = event.task?.metadata?.sampleId;
      if (sampleId != null) {
        const nextStatus: SampleStatus =
          event.type === 'task:completed' ? 'completed' : 'dismissed';
        setTerminalStatus((prev) => ({ ...prev, [String(sampleId)]: nextStatus }));
        showToast(
          event.type === 'task:completed'
            ? 'Task completed on desktop — synced to website'
            : 'Task dismissed on desktop — synced to website',
        );
      }
    }

    if (event.type === 'task:updated') {
      showToast('Sticky note updated — synced to website');
    }
  });

  const getSampleStatus = (sampleId: string): SampleStatus => {
    if (terminalStatus[sampleId]) return terminalStatus[sampleId];
    return getTaskByMetadata('sampleId', sampleId) ? 'on-desktop' : 'pending';
  };

  const acceptTask = async (task: (typeof SAMPLE_TASKS)[0]) => {
    await createTask({
      title: task.title,
      body: task.body,
      priority: task.priority,
      color: '#FFE066',
      metadata: { sampleId: task.id },
    });
    showToast('Sticky note created on your desktop!');
  };

  const completeFromWebsite = async (sampleId: string) => {
    const agentTask = getTaskByMetadata('sampleId', sampleId);
    if (!agentTask) return;
    await completeTask(agentTask.id);
    setTerminalStatus((prev) => ({ ...prev, [sampleId]: 'completed' }));
    showToast('Task completed from website — sticky closed on desktop');
  };

  const dismissFromWebsite = async (sampleId: string) => {
    const agentTask = getTaskByMetadata('sampleId', sampleId);
    if (!agentTask) return;
    await deleteTask(agentTask.id);
    setTerminalStatus((prev) => ({ ...prev, [sampleId]: 'dismissed' }));
    showToast('Task dismissed from website — sticky removed on desktop');
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
          <h1 className="text-xl font-bold">Acme Task Manager</h1>
          <p className="text-sm text-gray-500">Two-way sync with desktop stickies via WebSocket</p>
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

      <main className="max-w-2xl mx-auto p-6">
        <h2 className="text-lg font-semibold mb-4">Incoming Tasks</h2>
        <ul className="space-y-3">
          {SAMPLE_TASKS.map((task) => {
            const sampleStatus = getSampleStatus(task.id);
            const onDesktop = sampleStatus === 'on-desktop';

            return (
              <li
                key={task.id}
                className="bg-white rounded-lg border p-4 flex items-start justify-between gap-4 shadow-sm"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="font-medium">{task.title}</h3>
                    <span
                      className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${STATUS_COLORS[sampleStatus]}`}
                    >
                      {STATUS_LABELS[sampleStatus]}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600">{task.body}</p>
                </div>

                <div className="flex flex-col gap-2 shrink-0">
                  {sampleStatus === 'pending' && (
                    <button
                      onClick={() => acceptTask(task)}
                      disabled={loading || !isConnected}
                      className="px-3 py-1.5 rounded-md bg-blue-600 text-white text-sm hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Accept Task
                    </button>
                  )}

                  {onDesktop && (
                    <>
                      <button
                        onClick={() => completeFromWebsite(task.id)}
                        disabled={loading || !isConnected}
                        className="px-3 py-1.5 rounded-md bg-green-600 text-white text-sm hover:bg-green-700 disabled:opacity-50"
                      >
                        Complete
                      </button>
                      <button
                        onClick={() => dismissFromWebsite(task.id)}
                        disabled={loading || !isConnected}
                        className="px-3 py-1.5 rounded-md bg-red-500 text-white text-sm hover:bg-red-600 disabled:opacity-50"
                      >
                        Dismiss
                      </button>
                    </>
                  )}
                </div>
              </li>
            );
          })}
        </ul>

        {tasks.length > 0 && (
          <p className="mt-6 text-xs text-gray-500">
            {tasks.length} active sticky note{tasks.length === 1 ? '' : 's'} synced with agent.
            Complete or dismiss from either the website or the desktop sticky.
          </p>
        )}

        {error && (
          <p className="mt-4 text-sm text-red-600">{error.message}</p>
        )}
      </main>

      {toast && (
        <div className="fixed bottom-6 right-6 bg-gray-900 text-white px-4 py-2 rounded-lg shadow-lg text-sm max-w-sm">
          {toast}
        </div>
      )}
    </div>
  );
}

export default function App() {
  return (
    <DTPFProvider config={DTPF_CONFIG}>
      <Dashboard />
    </DTPFProvider>
  );
}
