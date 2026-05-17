import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { taskList, taskSetStatus, taskCreate } from '../ipc/invoke';
import type { Task } from '../ipc/types';

const SOURCE_LABEL: Record<Task['source'], string> = {
  planned: '',
  agent_discovered: '🤖',
  user_added: '👤',
};

interface Props {
  issueId: number;
}

export function TaskChecklist({ issueId }: Props) {
  const qc = useQueryClient();
  const [newTitle, setNewTitle] = useState('');
  const { data: tasks = [] } = useQuery({
    queryKey: ['tasks', issueId],
    queryFn: () => taskList(issueId),
  });

  const toggle = useMutation({
    mutationFn: ({ id, status }: { id: number; status: string }) =>
      taskSetStatus(id, status),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks', issueId] }),
  });

  const create = useMutation({
    mutationFn: (title: string) => taskCreate({ issue_id: issueId, title }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['tasks', issueId] });
      setNewTitle('');
    },
    onError: (err) => toast.error(`태스크 생성 실패: ${err}`),
  });

  const done = tasks.filter((t) => t.status === 'finished').length;

  function submitNew() {
    const t = newTitle.trim();
    if (t.length === 0) return;
    create.mutate(t);
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">
          태스크
        </span>
        <span className="text-xs text-slate-400">{done}/{tasks.length}</span>
      </div>
      {tasks.length === 0 && (
        <p className="text-xs text-slate-400">태스크 없음</p>
      )}
      {tasks.map((task) => {
        const finished = task.status === 'finished';
        return (
          <label key={task.id} className="flex items-start gap-2 cursor-pointer group">
            <input
              type="checkbox"
              checked={finished}
              onChange={() =>
                toggle.mutate({ id: task.id, status: finished ? 'ready' : 'finished' })
              }
              className="mt-0.5 rounded border-slate-300 text-indigo-600"
            />
            <span className={`text-sm flex-1 ${finished ? 'line-through text-slate-400' : 'text-slate-700'}`}>
              {task.title}
              {SOURCE_LABEL[task.source] && (
                <span className="ml-1 text-xs opacity-60">{SOURCE_LABEL[task.source]}</span>
              )}
            </span>
          </label>
        );
      })}
      <div className="flex gap-2 mt-2 pt-2 border-t border-slate-100">
        <input
          value={newTitle}
          onChange={(e) => setNewTitle(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') submitNew(); }}
          placeholder="새 태스크…"
          className="flex-1 px-2 py-1 text-sm border border-slate-200 rounded focus:outline-none focus:border-indigo-400"
        />
        <button
          type="button"
          onClick={submitNew}
          disabled={newTitle.trim().length === 0 || create.isPending}
          className="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded disabled:opacity-50"
        >
          추가
        </button>
      </div>
    </div>
  );
}
