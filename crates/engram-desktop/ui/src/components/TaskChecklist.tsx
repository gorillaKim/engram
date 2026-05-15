import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { taskList, taskSetStatus } from '../ipc/invoke';
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
  const { data: tasks = [] } = useQuery({
    queryKey: ['tasks', issueId],
    queryFn: () => taskList(issueId),
  });

  const toggle = useMutation({
    mutationFn: ({ id, status }: { id: number; status: string }) =>
      taskSetStatus(id, status),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks', issueId] }),
  });

  const done = tasks.filter((t) => t.status === 'finished').length;

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
    </div>
  );
}
