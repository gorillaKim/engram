import React, { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { toast } from 'sonner';
import { getPromptSettings } from '../ipc/invoke';

interface Props {
  type: 'issue' | 'epic' | 'mission' | 'retrospective';
  id: number;
  title: string;
  goal?: string | null;
  size?: 'xs' | 'sm' | 'md';
  className?: string;
}

export function PromptButton({
  type,
  id,
  title,
  goal,
  size = 'sm',
  className = '',
}: Props) {
  const [showTooltip, setShowTooltip] = useState(false);
  const [coords, setCoords] = useState<{ top?: number; bottom?: number; left: number } | null>(null);
  const [promptSettings, setPromptSettingsState] = useState<{
    issue_template: string;
    epic_template: string;
    mission_template: string;
    retrospective_template: string;
  }>({
    issue_template: '{{base prompt}}',
    epic_template: '{{base prompt}}',
    mission_template: '{{base prompt}}',
    retrospective_template: '{{base prompt}}',
  });
  const buttonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    getPromptSettings()
      .then((s) => {
        setPromptSettingsState({
          issue_template: s.issue_template || '{{base prompt}}',
          epic_template: s.epic_template || '{{base prompt}}',
          mission_template: s.mission_template || '{{base prompt}}',
          retrospective_template: s.retrospective_template || '{{base prompt}}',
        });
      })
      .catch(() => {});
  }, []);

  let basePromptText = '';
  if (type === 'issue') {
    basePromptText = `[engram issue-#${id}] "${title}" 이슈 작업을 진행해줘.`;
    if (goal && goal.trim()) {
      basePromptText += ` (목표: ${goal.trim()})`;
    }
  } else if (type === 'epic') {
    basePromptText = `[engram epic-#${id}] "${title}" 에픽 하위 이슈 작업을 진행해줘.`;
  } else if (type === 'mission') {
    basePromptText = `[engram mission-#${id}] "${title}" 미션 작업을 진행해줘.`;
  } else {
    basePromptText = `[engram retrospective-#${id}] "${title}" 회고 내용 및 액션 아이템 조치 사항을 확인하고 리뷰해줘.`;
  }

  const template =
    type === 'issue'
      ? promptSettings.issue_template
      : type === 'epic'
      ? promptSettings.epic_template
      : type === 'mission'
      ? promptSettings.mission_template
      : promptSettings.retrospective_template;

  const promptText = (template || '{{base prompt}}')
    .split('{{base prompt}}').join(basePromptText)
    .split('{{id}}').join(String(id))
    .split('{{title}}').join(title)
    .split('{{goal}}').join(goal || '')
    .split('{{type}}').join(type);

  const updateCoords = () => {
    if (!buttonRef.current) return;
    const rect = buttonRef.current.getBoundingClientRect();
    const tooltipWidth = 340;
    const tooltipHeight = 160;

    let left = rect.left + rect.width / 2 - tooltipWidth / 2;
    if (left + tooltipWidth > window.innerWidth - 16) {
      left = window.innerWidth - tooltipWidth - 16;
    }
    if (left < 16) {
      left = 16;
    }

    const spaceBelow = window.innerHeight - rect.bottom;
    if (spaceBelow < tooltipHeight && rect.top > tooltipHeight) {
      setCoords({
        bottom: window.innerHeight - rect.top + 6,
        left,
      });
    } else {
      setCoords({
        top: rect.bottom + 6,
        left,
      });
    }
  };

  const handleMouseEnter = () => {
    updateCoords();
    setShowTooltip(true);
  };

  const handleMouseLeave = () => {
    setShowTooltip(false);
  };

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
    <>
      <button
        ref={buttonRef}
        type="button"
        onClick={handleCopyPrompt}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        className={`inline-flex items-center gap-1 font-semibold rounded-md bg-indigo-50 hover:bg-indigo-100 text-indigo-700 border border-indigo-200/80 shadow-2xs transition-all cursor-pointer hover:scale-105 active:scale-95 shrink-0 ${sizeClass} ${className}`}
      >
        <span className="text-[11px] leading-none">⚡</span>
        <span className="leading-none">프롬프트</span>
      </button>

      {/* Portal Tooltip */}
      {showTooltip && coords && createPortal(
        <div
          style={{
            position: 'fixed',
            left: `${coords.left}px`,
            ...(coords.top != null ? { top: `${coords.top}px` } : { bottom: `${coords.bottom}px` }),
          }}
          className="w-[340px] max-w-[calc(100vw-32px)] p-2.5 bg-slate-900/95 backdrop-blur-md text-white text-[11px] leading-snug rounded-lg shadow-2xl z-[99999] pointer-events-none transition-all duration-150 animate-in fade-in zoom-in-95"
        >
          <div className="font-semibold text-indigo-300 mb-1 flex items-center justify-between">
            <span>⚡ 작업 프롬프트</span>
            <span className="text-[9px] text-slate-400 font-normal">클릭하여 복사</span>
          </div>
          <div className="font-mono bg-slate-800/90 p-2 rounded border border-slate-700/60 break-words whitespace-pre-wrap text-slate-200 text-[10.5px] max-h-48 overflow-y-auto">
            {promptText}
          </div>
        </div>,
        document.body
      )}
    </>
  );
}

