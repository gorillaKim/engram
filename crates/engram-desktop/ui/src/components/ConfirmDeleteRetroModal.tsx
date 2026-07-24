import { AlertTriangle, Trash2, X } from 'lucide-react';

interface Props {
  title: string;
  onConfirm: () => void;
  onClose: () => void;
}

export function ConfirmDeleteRetroModal({ title, onConfirm, onClose }: Props) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/50 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-2xl max-w-md w-full p-6 shadow-2xl border border-slate-200 flex flex-col gap-4 animate-in zoom-in-95 duration-150">
        <div className="flex items-center justify-between">
          <div className="p-3 rounded-full bg-rose-50 text-rose-600 border border-rose-100">
            <AlertTriangle className="w-6 h-6" />
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-lg text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex flex-col gap-1">
          <h3 className="text-lg font-bold text-slate-900">회고를 삭제하시겠습니까?</h3>
          <p className="text-xs text-slate-500 leading-relaxed">
            &quot;<span className="font-semibold text-slate-800">{title}</span>&quot; 회고 및 이에 포함된 모든 액션 아이템이 영구적으로 삭제됩니다. 이 작업은 되돌릴 수 없습니다.
          </p>
        </div>

        <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100 mt-1">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-xl border border-slate-200 text-slate-700 text-xs font-semibold hover:bg-slate-50 transition-all cursor-pointer"
          >
            취소
          </button>
          <button
            onClick={onConfirm}
            className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-rose-600 hover:bg-rose-700 text-white text-xs font-semibold shadow-sm transition-all cursor-pointer"
          >
            <Trash2 className="w-4 h-4" />
            <span>삭제하기</span>
          </button>
        </div>
      </div>
    </div>
  );
}
