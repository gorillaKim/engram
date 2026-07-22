import React, { useState } from 'react';
import { toast } from 'sonner';

interface Props {
  type: 'issue' | 'epic' | 'mission';
  id: number;
  title: string;
  goal?: string | null;
  size?: 'xs' | 'sm' | 'md';
  className?: string;
}

export function PromptButton({ type, id, title, goal, size = 'sm', className = '' }: Props) {
  const [showTooltip, setShowTooltip] = useState(false);

  let promptText = '';
  if (type === 'issue') {
    promptText = `[engram issue-#${id}] "${title}" 이슈 작업을 진행해줘.`;
    if (goal && goal.trim()) {
      promptText += ` (목표: ${goal.trim()})`;
    }
  } else if (type === 'epic') {
    promptText = `[engram epic-#${id}] "${title}" 에픽 하위 이슈 작업을 진행해줘.`;
  } else {
    promptText = `[engram mission-#${id}] "${title}" 미션 작업을 진행해줘.`;
  }

  const handleCopyPrompt = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(promptText);
    toast.success('작업 프롬프트가 클립보드에 복사되었습니다!');
  };

  const sizeClass = size === 'xs'
    ? 'px-1.5 py-0.5 text-[10px]'
    : size === 'sm'
    ? 'px-2 py-1 text-xs'
    : 'px-2.5 py-1.5 text-xs';

  return (
    <div
      className="relative inline-block"
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      <button
        type="button"
        onClick={handleCopyPrompt}
        className={`inline-flex items-center gap-1 font-semibold rounded-md bg-indigo-50 hover:bg-indigo-100 text-indigo-700 border border-indigo-200/80 shadow-2xs transition-all cursor-pointer hover:scale-105 active:scale-95 ${sizeClass} ${className}`}
      >
        <span className="text-[11px]">⚡</span>
        <span>Prompt</span>
      </button>

      {/* Hover Tooltip */}
      {showTooltip && (
        <div className="absolute left-1/2 -translate-x-1/2 bottom-full mb-2 w-64 p-2.5 bg-slate-900/95 backdrop-blur text-white text-[11px] leading-snug rounded-lg shadow-xl z-50 pointer-events-none transition-all duration-150 animate-in fade-in zoom-in-95">
          <div className="font-semibold text-indigo-300 mb-1 flex items-center justify-between">
            <span>⚡ 작업 프롬프트</span>
            <span className="text-[9px] text-slate-400 font-normal">클릭하여 복사</span>
          </div>
          <div className="font-mono bg-slate-800/90 p-2 rounded border border-slate-700/60 break-words whitespace-pre-wrap text-slate-200 text-[10.5px]">
            {promptText}
          </div>
          {/* Tooltip Arrow */}
          <div className="absolute left-1/2 -translate-x-1/2 top-full w-0 h-0 border-x-4 border-x-transparent border-t-4 border-t-slate-900/95" />
        </div>
      )}
    </div>
  );
}
