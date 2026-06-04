import { useState, useRef, useEffect } from 'react';
import type { IssueStatus, SprintStatus, EpicStatus, MissionStatus } from '../ipc/types';

interface StatusBadgeProps {
  status: IssueStatus | SprintStatus | EpicStatus | MissionStatus | string;
  type?: 'issue' | 'sprint' | 'epic' | 'mission' | 'task';
  variant?: 'ko' | 'en';
  className?: string;
  onChange?: (newStatus: string) => void;
  disabled?: boolean;
}

const ISSUE_STATUS_MAP: Record<string, { ko: string; en: string; cls: string }> = {
  required: { ko: '미진행', en: 'Required', cls: 'bg-slate-100 text-slate-600 border border-slate-200/50' },
  ready: { ko: '준비 완료', en: 'Ready', cls: 'bg-blue-50 text-blue-700 border border-blue-100' },
  working: { ko: '진행 중', en: 'Working', cls: 'bg-indigo-50 text-indigo-700 border border-indigo-100' },
  demo: { ko: '데모', en: 'Demo', cls: 'bg-amber-50 text-amber-700 border border-amber-100' },
  finished: { ko: '완료', en: 'Finished', cls: 'bg-emerald-50 text-emerald-700 border border-emerald-100' },
  cancelled: { ko: '취소', en: 'Cancelled', cls: 'bg-red-50 text-red-600 border border-red-100' },
};

const SPRINT_STATUS_MAP: Record<string, { ko: string; en: string; cls: string }> = {
  planning: { ko: '계획', en: 'Planning', cls: 'bg-yellow-100 text-yellow-700' },
  active: { ko: '활성', en: 'Active', cls: 'bg-green-100 text-green-700' },
  completed: { ko: '완료', en: 'Completed', cls: 'bg-slate-100 text-slate-500' },
  cancelled: { ko: '취소', en: 'Cancelled', cls: 'bg-red-50 text-red-400' },
};

const EPIC_MISSION_STATUS_MAP: Record<string, { ko: string; en: string; cls: string }> = {
  active: { ko: '활성', en: 'Active', cls: 'bg-emerald-100 text-emerald-700 border border-emerald-200/30' },
  completed: { ko: '완료', en: 'Completed', cls: 'bg-slate-100 text-slate-500' },
  cancelled: { ko: '취소', en: 'Cancelled', cls: 'bg-red-100 text-red-600 border border-red-200/30' },
};

export function StatusBadge({
  status,
  type,
  variant = 'en',
  className = '',
  onChange,
  disabled = false,
}: StatusBadgeProps) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  const normStatus = status.toLowerCase();

  // 1. Determine which map to use based on type or status key heuristics
  let config = ISSUE_STATUS_MAP[normStatus];
  let statusMap = ISSUE_STATUS_MAP;

  if (type === 'sprint' || (SPRINT_STATUS_MAP[normStatus] && type !== 'issue' && type !== 'task')) {
    statusMap = SPRINT_STATUS_MAP;
    if (normStatus === 'active' || normStatus === 'completed' || normStatus === 'cancelled') {
      config = type === 'sprint' ? SPRINT_STATUS_MAP[normStatus] : (EPIC_MISSION_STATUS_MAP[normStatus] || SPRINT_STATUS_MAP[normStatus]);
    } else {
      config = SPRINT_STATUS_MAP[normStatus];
    }
  } else if (type === 'epic' || type === 'mission' || EPIC_MISSION_STATUS_MAP[normStatus]) {
    statusMap = EPIC_MISSION_STATUS_MAP;
    config = EPIC_MISSION_STATUS_MAP[normStatus];
  }

  // Fallback if not matching any
  if (!config) {
    config = {
      ko: status,
      en: status.toUpperCase(),
      cls: 'bg-slate-100 text-slate-600 border border-slate-200/50',
    };
  }

  const label = variant === 'ko' ? config.ko : config.en;
  const isInteractive = onChange != null && !disabled;

  return (
    <span
      ref={containerRef}
      className={`relative inline-block ${isInteractive ? 'cursor-pointer select-none' : ''}`}
      onClick={(e) => {
        if (isInteractive) {
          e.stopPropagation();
          setIsOpen(!isOpen);
        }
      }}
    >
      <span
        className={`inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10px] font-bold uppercase tracking-wide shrink-0 transition-colors ${config.cls} ${isInteractive ? 'hover:brightness-95 active:brightness-90' : ''} ${className}`}
      >
        {label}
        {isInteractive && <span className="text-[9px] opacity-60">▾</span>}
      </span>

      {isOpen && isInteractive && (
        <span
          className="absolute left-0 mt-1 z-50 min-w-[100px] py-1 bg-white border border-slate-200 rounded-md shadow-lg flex flex-col cursor-default"
          onClick={(e) => e.stopPropagation()}
        >
          {Object.entries(statusMap).map(([sKey, sConf]) => (
            <button
              key={sKey}
              type="button"
              onClick={() => {
                onChange(sKey);
                setIsOpen(false);
              }}
              className={`text-left px-2.5 py-1 text-[11px] hover:bg-slate-50 transition-colors w-full font-medium ${
                normStatus === sKey ? 'text-indigo-600 font-semibold bg-indigo-50/40' : 'text-slate-600'
              }`}
            >
              {variant === 'ko' ? sConf.ko : sConf.en}
            </button>
          ))}
        </span>
      )}
    </span>
  );
}
