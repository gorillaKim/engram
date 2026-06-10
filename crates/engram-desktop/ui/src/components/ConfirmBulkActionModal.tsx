import { BaseModal } from './BaseModal';

interface ConfirmBulkActionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  description: string;
  items: { id: number; title: string }[];
  confirmText?: string;
  isPending?: boolean;
}

export function ConfirmBulkActionModal({
  isOpen,
  onClose,
  onConfirm,
  title,
  description,
  items,
  confirmText = '확인',
  isPending = false,
}: ConfirmBulkActionModalProps) {
  return (
    <BaseModal open={isOpen} onClose={onClose} title={title} maxWidth="max-w-md">
      <div className="space-y-4">
        <p className="text-xs text-slate-400">
          {description}
        </p>

        {items.length > 0 && (
          <div className="max-h-40 overflow-y-auto border border-slate-700 rounded-lg bg-slate-800/40 p-3 space-y-1.5">
            <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block mb-1">
              대상 목록 ({items.length}개)
            </label>
            {items.map((item) => (
              <div key={item.id} className="flex items-center justify-between text-xs text-slate-300">
                <span className="truncate max-w-[280px]">· {item.title}</span>
                <span className="font-mono text-slate-500">#{item.id}</span>
              </div>
            ))}
          </div>
        )}

        <div className="flex justify-end gap-2 pt-4 border-t border-slate-800 mt-6">
          <button
            type="button"
            onClick={onClose}
            disabled={isPending}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded-lg transition-colors"
          >
            취소
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={isPending || items.length === 0}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-lg disabled:opacity-50 font-medium transition-colors"
          >
            {isPending ? '처리 중…' : confirmText}
          </button>
        </div>
      </div>
    </BaseModal>
  );
}
