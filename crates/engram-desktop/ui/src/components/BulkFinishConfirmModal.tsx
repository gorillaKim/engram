import type { Issue } from '../ipc/types';
import { BaseModal } from './BaseModal';

interface Props {
  open: boolean;
  issues: Issue[];
  projectKey: string;
  onConfirm: () => void;
  onCancel: () => void;
  isPending: boolean;
}

export function BulkFinishConfirmModal({
  open, issues, projectKey, onConfirm, onCancel, isPending,
}: Props) {
  return (
    <BaseModal open={open} onClose={onCancel} title="DEMO 일괄 완료">
      <div className="flex flex-col gap-4">
        <p className="text-sm text-slate-400">
          <span className="font-semibold text-indigo-400">{projectKey}</span> 프로젝트의 DEMO 이슈{' '}
          <span className="font-bold text-white">{issues.length}</span>건을 완료 처리합니다.
        </p>

        <div className="bg-slate-800 rounded-lg border border-slate-700 max-h-[240px] overflow-y-auto">
          {issues.map((issue, idx) => (
            <div
              key={issue.id}
              className={`flex items-center gap-3 px-3 py-2.5 ${
                idx < issues.length - 1 ? 'border-b border-slate-700/60' : ''
              }`}
            >
              <span className="text-[11px] font-mono text-slate-500 flex-shrink-0">#{issue.id}</span>
              <span className="text-sm text-slate-300 truncate">{issue.title}</span>
              <span className="ml-auto text-[10px] font-semibold text-amber-400 bg-amber-900/30 px-1.5 py-0.5 rounded flex-shrink-0">
                DEMO
              </span>
              <span className="text-[10px] text-emerald-400 flex-shrink-0">→ FINISHED</span>
            </div>
          ))}
        </div>

        <div className="flex items-center justify-end gap-2 pt-1">
          <button
            type="button"
            onClick={onCancel}
            disabled={isPending}
            className="px-4 py-2 text-sm text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors"
          >
            취소
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={isPending}
            className="px-4 py-2 text-sm font-semibold text-white bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors flex items-center gap-2"
          >
            {isPending ? (
              <>
                <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                처리 중…
              </>
            ) : (
              <>✓ {issues.length}건 완료 처리</>
            )}
          </button>
        </div>
      </div>
    </BaseModal>
  );
}
