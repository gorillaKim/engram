import React, { useState, useEffect, useRef, useMemo } from 'react';
import { BarChart3, FileText, Link, PieChart, Sparkles } from 'lucide-react';

export interface SlashMenuItem {
  key: string;
  label: string;
  description: string;
  icon: React.ReactNode;
  action: () => void;
}

interface SlashMenuPluginProps {
  onSelect: (key: string) => void;
  onClose: () => void;
  position: { top: number; left: number };
}

export function SlashMenuPlugin({ onSelect, onClose, position }: SlashMenuPluginProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const menuRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const items: SlashMenuItem[] = useMemo(
    () => [
      {
        key: 'sprint-stats',
        label: '/sprint-stats',
        description: '이번 스프린트 총 이슈 및 완료율 통계 요약 삽입',
        icon: <BarChart3 className="w-4 h-4 text-indigo-600 shrink-0" />,
        action: () => onSelect('sprint-stats'),
      },
      {
        key: 'analyze',
        label: '/analyze',
        description: '이번 스프린트 각 에픽별 현황 및 진행률 분석 표 추가',
        icon: <PieChart className="w-4 h-4 text-purple-600 shrink-0" />,
        action: () => onSelect('analyze'),
      },
      {
        key: 'issue-link',
        label: '/issue',
        description: '스프린트 이슈 선택 팝업 열기 및 링크 칩 삽입',
        icon: <Link className="w-4 h-4 text-blue-600 shrink-0" />,
        action: () => onSelect('issue-link'),
      },
      {
        key: 'kpt-template',
        label: '/kpt-template',
        description: 'Keep / Problem / Try 회고 템플릿 서식 생성',
        icon: <FileText className="w-4 h-4 text-emerald-600 shrink-0" />,
        action: () => onSelect('kpt-template'),
      },
    ],
    [onSelect]
  );

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        e.stopPropagation();
        setSelectedIndex((prev) => (prev + 1) % items.length);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        e.stopPropagation();
        setSelectedIndex((prev) => (prev - 1 + items.length) % items.length);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        e.stopPropagation();
        if (items[selectedIndex]) {
          items[selectedIndex].action();
        }
      } else if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        onClose();
      }
    }

    window.addEventListener('keydown', handleKeyDown, true);
    return () => window.removeEventListener('keydown', handleKeyDown, true);
  }, [selectedIndex, items, onClose]);

  useEffect(() => {
    const el = itemRefs.current[selectedIndex];
    if (el) {
      el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }, [selectedIndex]);

  return (
    <div
      ref={menuRef}
      style={{ top: `${position.top}px`, left: `${position.left}px` }}
      className="fixed z-50 w-72 rounded-xl border border-slate-200 bg-white/95 p-2 shadow-2xl backdrop-blur-md flex flex-col gap-1"
    >
      <div className="px-2 py-1 text-[11px] font-semibold tracking-wider text-slate-400 uppercase flex items-center gap-1.5 select-none border-b border-slate-100 mb-0.5 pb-1.5">
        <Sparkles className="w-3.5 h-3.5 text-indigo-600 shrink-0" />
        <span>Slash Commands</span>
      </div>
      <div className="flex flex-col gap-1 max-h-60 overflow-y-auto p-1">
        {items.map((item, idx) => (
          <button
            key={item.key}
            ref={(el) => (itemRefs.current[idx] = el)}
            onClick={item.action}
            onMouseMove={() => {
              if (selectedIndex !== idx) {
                setSelectedIndex(idx);
              }
            }}
            className={`flex items-start gap-2.5 px-3 py-2 rounded-lg text-left transition-all duration-150 select-none box-border ${
              idx === selectedIndex
                ? 'bg-indigo-50/90 text-indigo-900 border border-indigo-300/80 shadow-xs font-medium'
                : 'text-slate-700 hover:bg-slate-50 border border-transparent'
            }`}
          >
            <div className="mt-0.5 p-1 rounded bg-slate-100 border border-slate-200 shrink-0">
              {item.icon}
            </div>
            <div className="flex flex-col gap-0.5 min-w-0 flex-1">
              <span className="text-xs font-semibold tracking-tight text-slate-900 truncate">
                {item.label}
              </span>
              <span className="text-[11px] text-slate-500 truncate leading-snug">
                {item.description}
              </span>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
