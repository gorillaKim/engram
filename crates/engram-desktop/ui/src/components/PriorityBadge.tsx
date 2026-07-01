import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import type { IssuePriority } from '../ipc/types';

interface PriorityBadgeProps {
  priority: IssuePriority;
  onChange?: (newPriority: IssuePriority) => void;
  disabled?: boolean;
}

const colors: Record<IssuePriority, string> = {
  critical: 'bg-red-500',
  high: 'bg-orange-500',
  medium: 'bg-amber-400',
  low: 'bg-slate-400',
};

const labels: Record<IssuePriority, string> = {
  critical: 'Critical',
  high: 'High',
  medium: 'Medium',
  low: 'Low',
};

export function PriorityBadge({ priority, onChange, disabled = false }: PriorityBadgeProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [popoverPos, setPopoverPos] = useState<{ top: number; left: number } | null>(null);
  const badgeRef = useRef<HTMLSpanElement>(null);
  const containerRef = useRef<HTMLSpanElement>(null);

  const isInteractive = onChange != null && !disabled;

  // Calculate popover position relative to badge (fixed coords)
  useEffect(() => {
    if (isOpen && badgeRef.current) {
      const rect = badgeRef.current.getBoundingClientRect();
      setPopoverPos({ top: rect.bottom + 6, left: rect.left });
    }
  }, [isOpen]);

  // Close on outside click
  useEffect(() => {
    if (!isOpen) return;
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        const target = e.target as Node;
        // Also check if clicking inside the portal popover
        const portal = document.getElementById('priority-badge-portal');
        if (portal && portal.contains(target)) return;
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  return (
    <span
      ref={containerRef}
      className={`relative inline-flex items-center ${isInteractive ? 'cursor-pointer select-none' : ''}`}
      onClick={(e) => {
        if (isInteractive) {
          e.stopPropagation();
          setIsOpen(!isOpen);
        }
      }}
    >
      <span
        ref={badgeRef}
        className="inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold border border-slate-200 bg-white shadow-sm hover:bg-slate-50 transition-colors shrink-0"
        title={`우선순위: ${priority}${isInteractive ? ' (클릭하여 수정)' : ''}`}
      >
        <span className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${colors[priority]}`} />
        <span className="text-[10px] text-slate-600 font-medium capitalize">{labels[priority]}</span>
        {isInteractive && <span className="text-[9px] opacity-60">▾</span>}
      </span>

      {isOpen && isInteractive && popoverPos &&
        createPortal(
          <span
            id="priority-badge-portal"
            style={{ position: 'fixed', top: popoverPos.top, left: popoverPos.left }}
            className="z-[9999] min-w-[96px] py-1 bg-white border border-slate-200 rounded-md shadow-lg flex flex-col cursor-default"
            onClick={(e) => e.stopPropagation()}
          >
            {(Object.keys(colors) as IssuePriority[]).map((pKey) => (
              <button
                key={pKey}
                type="button"
                onClick={() => {
                  onChange(pKey);
                  setIsOpen(false);
                }}
                className={`text-left px-2.5 py-1 text-[11px] hover:bg-slate-50 transition-colors w-full font-medium flex items-center gap-1.5 ${
                  priority === pKey ? 'text-indigo-600 font-semibold bg-indigo-50/40' : 'text-slate-600'
                }`}
              >
                <span className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${colors[pKey]}`} />
                {labels[pKey]}
              </button>
            ))}
          </span>,
          document.body
        )
      }
    </span>
  );
}
