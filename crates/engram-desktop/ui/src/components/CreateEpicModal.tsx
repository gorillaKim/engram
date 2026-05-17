import { useState, useEffect } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicCreate, sprintCurrent } from '../ipc/invoke';

interface Props {
  open: boolean;
  onClose: () => void;
  defaultProjectKey?: string;
}

export function CreateEpicModal({ open, onClose, defaultProjectKey }: Props) {
  const qc = useQueryClient();
  const { data: sprint } = useQuery({
    queryKey: ['sprintCurrent'],
    queryFn: () => sprintCurrent(),
    enabled: open,
  });

  const [projectKey, setProjectKey] = useState('');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');

  useEffect(() => {
    if (open) {
      setProjectKey(defaultProjectKey ?? '');
      setTitle('');
      setDescription('');
    }
  }, [open, defaultProjectKey]);

  const create = useMutation({
    mutationFn: () => {
      if (!sprint) throw new Error('활성 스프린트가 없습니다');
      return epicCreate({
        sprint_id: sprint.id,
        project_key: projectKey.trim(),
        title: title.trim(),
        description: description.trim() || undefined,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽이 생성되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`에픽 생성 실패: ${err}`),
  });

  if (!open) return null;

  const canSubmit = title.trim().length > 0 && projectKey.trim().length > 0 && !!sprint;

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-slate-900 border border-slate-700 rounded-lg p-6 w-full max-w-md mx-4">
        <h3 className="text-lg font-semibold text-white mb-4">새 에픽 생성</h3>

        {!sprint && (
          <p className="text-xs text-amber-300 mb-3">활성 스프린트가 없습니다. CLI에서 sprint를 active로 만드세요.</p>
        )}

        <div className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">프로젝트 키 *</label>
            <input
              value={projectKey}
              onChange={(e) => setProjectKey(e.target.value)}
              placeholder="예: xpert-da-web"
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">에픽 이름 *</label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="에픽 이름"
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              placeholder="에픽 설명 (선택)"
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          {sprint && (
            <p className="text-xs text-slate-500">스프린트: {sprint.name}</p>
          )}
        </div>

        <div className="flex gap-2 justify-end mt-6">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded-lg"
          >
            취소
          </button>
          <button
            type="button"
            disabled={!canSubmit || create.isPending}
            onClick={() => create.mutate()}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-lg disabled:opacity-50"
          >
            {create.isPending ? '생성 중…' : '생성'}
          </button>
        </div>
      </div>
    </div>
  );
}
