import React from 'react';

interface BaseModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  maxWidth?: string; // 예: 'max-w-md', 'max-w-lg', 'max-w-xl'
}

export function BaseModal({
  open,
  onClose,
  title,
  children,
  maxWidth = 'max-w-md',
}: BaseModalProps) {
  if (!open) return null;

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        className={`bg-slate-900 border border-slate-700 rounded-lg p-6 w-full ${maxWidth} mx-4 flex flex-col max-h-[90vh] overflow-y-auto`}
      >
        <div className="flex items-center justify-between mb-4 flex-shrink-0">
          <h3 className="text-lg font-semibold text-white">{title}</h3>
          <button
            type="button"
            onClick={onClose}
            className="text-slate-400 hover:text-white transition-colors p-1 rounded-lg hover:bg-slate-800"
            title="닫기"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2}
              stroke="currentColor"
              className="w-5 h-5"
            >
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div className="flex-1 min-h-0">
          {children}
        </div>
      </div>
    </div>
  );
}
