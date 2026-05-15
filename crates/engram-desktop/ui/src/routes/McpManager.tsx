import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { useMcpStatus } from '../hooks/useMcpStatus';
import { useMcpCalls } from '../hooks/useMcpCalls';
import { useMcpLogs } from '../hooks/useMcpLogs';
import { mcpStart, mcpStop, mcpRestart, mcpSetAutostart } from '../ipc/invoke';
import type { SupervisorStatusSnapshot } from '../ipc/types';

function formatUptime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

function LevelBadge({ level }: { level: string }) {
  const cls =
    level === 'ERROR' ? 'text-red-500' :
    level === 'WARN'  ? 'text-amber-500' :
    level === 'INFO'  ? 'text-blue-500' :
    'text-slate-400';
  return <span className={`font-mono text-xs font-bold ${cls}`}>{level.padEnd(5)}</span>;
}

export function McpManager() {
  const qc = useQueryClient();
  const { data: status, isLoading } = useMcpStatus();
  const calls = useMcpCalls();
  const logs = useMcpLogs();
  const [port, setPort] = useState(3456);
  const [showModal, setShowModal] = useState(false);

  const mutationOpts = {
    onSuccess: (snap: SupervisorStatusSnapshot) => {
      qc.setQueryData(['mcpStatus'], snap);
    },
    onError: (err: unknown) => toast.error(`MCP 오류: ${err}`),
  };

  const startMut   = useMutation({ mutationFn: () => mcpStart(port), ...mutationOpts });
  const stopMut    = useMutation({ mutationFn: mcpStop, ...mutationOpts });
  const restartMut = useMutation({ mutationFn: () => mcpRestart(port), ...mutationOpts });
  const autostartMut = useMutation({
    mutationFn: (on: boolean) => mcpSetAutostart(on),
  });

  const endpoint = `http://127.0.0.1:${status?.port ?? port}/mcp`;

  if (isLoading) return <div className="p-8 text-slate-400">Loading…</div>;

  return (
    <div className="flex flex-col gap-4 p-6 max-w-3xl mx-auto overflow-auto h-full">
      {/* Status panel */}
      <div className="bg-white border border-slate-200 rounded-lg p-4 flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className={`w-2.5 h-2.5 rounded-full ${status?.running ? 'bg-green-500' : 'bg-slate-400'}`} />
            <span className="text-sm font-semibold text-slate-700">
              {status?.running ? 'Running' : 'Stopped'}
            </span>
            {status?.running && (
              <span className="text-xs text-slate-400">uptime {formatUptime(status.uptime_secs)}</span>
            )}
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-500">
            <code className="bg-slate-100 px-2 py-0.5 rounded">{endpoint}</code>
            <button
              onClick={() => navigator.clipboard.writeText(endpoint)}
              className="text-slate-400 hover:text-slate-600"
              title="복사"
            >📋</button>
            <button
              onClick={() => setShowModal(true)}
              className="text-indigo-500 hover:text-indigo-700 text-xs"
            >
              Claude Code 설정 보기
            </button>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <label className="text-xs text-slate-500">포트</label>
          <input
            type="number"
            min={1024}
            max={65535}
            value={port}
            onChange={e => {
              const val = Number(e.target.value);
              if (val >= 1024 && val <= 65535) setPort(val);
            }}
            className="w-24 text-sm border border-slate-300 rounded px-2 py-1 text-center"
          />
          <label className="flex items-center gap-1.5 text-xs text-slate-600 cursor-pointer ml-auto">
            <input
              type="checkbox"
              checked={status?.autostart ?? true}
              onChange={e => autostartMut.mutate(e.target.checked)}
              className="rounded border-slate-300 text-indigo-600"
            />
            앱 시작 시 자동 기동
          </label>
        </div>

        <div className="flex gap-2">
          <button
            onClick={() => startMut.mutate()}
            disabled={status?.running || startMut.isPending}
            className="px-3 py-1.5 text-xs rounded bg-green-600 text-white hover:bg-green-700 disabled:opacity-40"
          >
            ▶ 시작
          </button>
          <button
            onClick={() => stopMut.mutate()}
            disabled={!status?.running || stopMut.isPending}
            className="px-3 py-1.5 text-xs rounded bg-red-600 text-white hover:bg-red-700 disabled:opacity-40"
          >
            ■ 정지
          </button>
          <button
            onClick={() => restartMut.mutate()}
            disabled={restartMut.isPending}
            className="px-3 py-1.5 text-xs rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-40"
          >
            ↻ 재시작
          </button>
        </div>
      </div>

      {/* Recent calls */}
      <div className="bg-white border border-slate-200 rounded-lg p-4">
        <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-3">
          최근 호출 ({calls.length})
        </h3>
        {calls.length === 0 ? (
          <p className="text-xs text-slate-400">호출 없음</p>
        ) : (
          <div className="overflow-auto max-h-48">
            <table className="w-full text-xs">
              <thead>
                <tr className="text-slate-400">
                  <th className="text-left font-medium pb-1">시각</th>
                  <th className="text-left font-medium pb-1">도구</th>
                  <th className="text-left font-medium pb-1">결과</th>
                  <th className="text-right font-medium pb-1">ms</th>
                </tr>
              </thead>
              <tbody>
                {calls.slice(0, 50).map((c, i) => (
                  <tr key={i} className="border-t border-slate-100">
                    <td className="py-0.5 text-slate-400 font-mono">
                      {new Date(c.ts).toLocaleTimeString()}
                    </td>
                    <td className="py-0.5 text-slate-700 font-mono">{c.name}</td>
                    <td className="py-0.5">
                      <span className={c.ok ? 'text-green-600' : 'text-red-500'}>
                        {c.ok ? 'ok' : c.reason ?? 'err'}
                      </span>
                    </td>
                    <td className="py-0.5 text-right text-slate-400 font-mono">{c.duration_ms}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Log tail */}
      <div className="bg-slate-900 rounded-lg p-4 text-xs font-mono">
        <h3 className="text-slate-400 uppercase tracking-wider text-xs mb-2">로그 (tail)</h3>
        <div className="overflow-auto max-h-48 flex flex-col gap-0.5">
          {logs.length === 0 && <span className="text-slate-500">로그 없음</span>}
          {logs.map((l, i) => (
            <div key={i} className="flex gap-2 text-slate-300">
              <LevelBadge level={l.level} />
              <span className="text-slate-500 truncate max-w-[180px]">{l.target}</span>
              <span>{l.msg}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Claude Code config modal */}
      {showModal && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          onClick={() => setShowModal(false)}
        >
          <div
            className="bg-white rounded-xl shadow-2xl p-6 max-w-md w-full mx-4 relative"
            onClick={e => e.stopPropagation()}
          >
            <h3 className="text-sm font-semibold mb-3">Claude Code 설정</h3>
            <p className="text-xs text-slate-500 mb-2">
              ~/.claude.json의 "mcpServers"에 붙여넣으세요 (Streamable HTTP):
            </p>
            <pre className="bg-slate-100 rounded p-3 text-xs overflow-auto">
{`"engram": {
  "type": "http",
  "url": "${endpoint}"
}`}
            </pre>
            <button
              onClick={() => {
                navigator.clipboard.writeText(`"engram": {\n  "type": "http",\n  "url": "${endpoint}"\n}`);
                toast.success('복사됨');
              }}
              className="mt-3 text-xs text-indigo-600 hover:text-indigo-800"
            >
              복사
            </button>
            <button
              onClick={() => setShowModal(false)}
              className="absolute top-4 right-4 text-slate-400 hover:text-slate-600"
            >
              ×
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
