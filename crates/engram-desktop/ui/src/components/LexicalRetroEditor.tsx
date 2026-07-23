import React, { useState, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { SlashMenuPlugin } from './SlashMenuPlugin';
import { IssueSelectModal, IssueOption } from './IssueSelectModal';
import { BarChart3, Edit3, Eye, ExternalLink, Sparkles } from 'lucide-react';
import { useUIStore } from '../store/ui';

interface LexicalRetroEditorProps {
  value: string;
  onChange: (val: string) => void;
  sprintStats?: {
    totalIssues: number;
    finishedIssues: number;
    completionRate: number;
  };
}

export function LexicalRetroEditor({
  value,
  onChange,
  sprintStats,
}: LexicalRetroEditorProps) {
  const { selectIssue } = useUIStore();
  const [mode, setMode] = useState<'edit' | 'preview'>('edit');
  const [showSlashMenu, setShowSlashMenu] = useState(false);
  const [showIssueModal, setShowIssueModal] = useState(false);
  const [menuPos, setMenuPos] = useState({ top: 180, left: 240 });
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const slashCursorPos = useRef<number>(0);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === '/') {
      const target = e.currentTarget;
      const cursor = target.selectionStart;
      if (cursor === 0 || value[cursor - 1] === '\n' || value[cursor - 1] === ' ') {
        slashCursorPos.current = cursor;
        setMenuPos({ top: 180, left: 240 });
        setShowSlashMenu(true);
      }
    }
  };

  const handleSlashSelect = (key: string) => {
    setShowSlashMenu(false);
    if (!textareaRef.current) return;

    const cursor = textareaRef.current.selectionStart;
    let beforeSlash = value.slice(0, cursor);
    if (beforeSlash.endsWith('/')) {
      beforeSlash = beforeSlash.slice(0, -1);
    }
    const afterSlash = value.slice(cursor);

    let insertion = '';

    if (key === 'sprint-stats') {
      const total = sprintStats?.totalIssues ?? 20;
      const finished = sprintStats?.finishedIssues ?? 17;
      const rate = sprintStats?.completionRate ?? 85;
      insertion = `> 📊 **스프린트 통계**: **${rate}%** (${finished}/${total} 이슈 완료)\n\n`;
      onChange(beforeSlash + insertion + afterSlash);
    } else if (key === 'analyse') {
      insertion = `### 📊 이번 스프린트 에픽별 현황 분석\n\n| 에픽명 | 전체 이슈 | 완료 | 진행률 |\n| :--- | :---: | :---: | :---: |\n| [Core & DB] | 10 | 10 | 100% |\n| [MCP & CLI] | 5 | 5 | 100% |\n| [Desktop UI] | 5 | 2 | 40% |\n\n`;
      onChange(beforeSlash + insertion + afterSlash);
    } else if (key === 'kpt-template') {
      insertion = `## 🟢 Keep (잘한 점 & 유지할 점)\n- \n\n## 🔴 Problem (아쉬운 점 & 문제점)\n- \n\n## 🟡 Try (시도할 개선 방향)\n- \n\n`;
      onChange(beforeSlash + insertion + afterSlash);
    } else if (key === 'issue-link') {
      setShowIssueModal(true);
    }
  };

  const handleIssueSelect = (issue: IssueOption) => {
    setShowIssueModal(false);
    if (!textareaRef.current) return;

    const cursor = textareaRef.current.selectionStart;
    let beforeSlash = value.slice(0, cursor);
    if (beforeSlash.endsWith('/')) {
      beforeSlash = beforeSlash.slice(0, -1);
    }
    const afterSlash = value.slice(cursor);

    const insertion = `[#${issue.id} ${issue.title}] `;
    onChange(beforeSlash + insertion + afterSlash);
  };

  // Custom Markdown renderers for elegant MD preview
  const MarkdownComponents = {
    h2: ({ children }: any) => {
      const str = String(children);
      let badgeBg = 'bg-slate-100 text-slate-800 border-slate-200';
      if (str.includes('Keep')) badgeBg = 'bg-emerald-50 text-emerald-800 border-emerald-200';
      if (str.includes('Problem')) badgeBg = 'bg-rose-50 text-rose-800 border-rose-200';
      if (str.includes('Try')) badgeBg = 'bg-amber-50 text-amber-800 border-amber-200';

      return (
        <h2 className={`text-base font-bold my-4 p-2.5 rounded-lg border flex items-center gap-2 ${badgeBg}`}>
          <span>{children}</span>
        </h2>
      );
    },
    h3: ({ children }: any) => (
      <h3 className="text-sm font-bold text-slate-800 my-3 flex items-center gap-1.5 pb-1 border-b border-slate-200">
        <Sparkles className="w-4 h-4 text-indigo-600" />
        <span>{children}</span>
      </h3>
    ),
    blockquote: ({ children }: any) => (
      <blockquote className="my-3 p-3 bg-indigo-50/70 border-l-4 border-indigo-500 rounded-r-lg text-indigo-950 font-medium text-xs shadow-2xs">
        {children}
      </blockquote>
    ),
    table: ({ children }: any) => (
      <div className="my-4 overflow-x-auto rounded-lg border border-slate-200 shadow-2xs">
        <table className="w-full text-xs text-left border-collapse">{children}</table>
      </div>
    ),
    thead: ({ children }: any) => <thead className="bg-slate-100 text-slate-700 font-semibold border-b border-slate-200">{children}</thead>,
    th: ({ children }: any) => <th className="px-3.5 py-2.5 border-r border-slate-200 last:border-r-0">{children}</th>,
    td: ({ children }: any) => <td className="px-3.5 py-2 border-t border-r border-slate-200 last:border-r-0 text-slate-700">{children}</td>,
    p: ({ children }: any) => {
      // [#1192 이슈제목] 텍스트 패턴 스마트 칩 렌더링
      const str = String(children);
      const match = str.match(/\[#(\d+)\s+([^\]]+)\]/);
      if (match) {
        const issueId = parseInt(match[1], 10);
        const title = match[2];
        return (
          <p className="my-2 text-sm leading-relaxed">
            <button
              onClick={() => selectIssue(issueId)}
              className="inline-flex items-center gap-1 px-2 py-0.5 mx-1 rounded bg-indigo-50 hover:bg-indigo-100 text-indigo-700 border border-indigo-200 text-xs font-mono font-medium transition-colors shadow-2xs cursor-pointer"
            >
              <span>#{issueId}</span>
              <span className="font-sans font-normal truncate max-w-xs">{title}</span>
              <ExternalLink className="w-3 h-3 text-indigo-500" />
            </button>
          </p>
        );
      }
      return <p className="my-2 text-sm leading-relaxed text-slate-700">{children}</p>;
    },
  };

  return (
    <div className="relative flex flex-col h-full bg-white border border-slate-200 rounded-xl overflow-hidden shadow-sm">
      {/* 툴바 (여백 확보 및 일그러짐 방지) */}
      <div className="flex items-center justify-between border-b border-slate-200 bg-slate-50 px-5 py-3 text-xs text-slate-600 shrink-0 gap-4 flex-wrap">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2.5">
            <span className="font-bold text-slate-800 text-sm">Lexical Rich Editor</span>
            <span className="px-2.5 py-1 rounded-full bg-indigo-50 text-indigo-700 text-[11px] font-mono font-medium border border-indigo-200/80">
              Slash (/) Enabled
            </span>
          </div>

          {/* Edit / Preview 토글 탭 */}
          <div className="flex items-center p-1 bg-slate-200/80 rounded-lg border border-slate-300/70">
            <button
              onClick={() => setMode('edit')}
              className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-semibold transition-all ${
                mode === 'edit'
                  ? 'bg-white text-indigo-600 shadow-sm border border-slate-200/80'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <Edit3 className="w-3.5 h-3.5" />
              <span>Edit</span>
            </button>
            <button
              onClick={() => setMode('preview')}
              className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-semibold transition-all ${
                mode === 'preview'
                  ? 'bg-white text-indigo-600 shadow-sm border border-slate-200/80'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <Eye className="w-3.5 h-3.5" />
              <span>Preview</span>
            </button>
          </div>
        </div>

        {sprintStats && (
          <div className="flex items-center gap-2 text-slate-700 bg-white px-3 py-1.5 rounded-lg border border-slate-200 shadow-2xs font-medium">
            <BarChart3 className="w-4 h-4 text-indigo-600 shrink-0" />
            <span>완료율: <strong className="text-emerald-600 font-bold text-sm">{sprintStats.completionRate}%</strong></span>
          </div>
        )}
      </div>

      {/* 에디터 / 아름다운 마크다운 미리보기 본문 */}
      <div className="relative flex-1 p-5 overflow-y-auto min-h-0 bg-white">
        {mode === 'edit' ? (
          <>
            <textarea
              ref={textareaRef}
              value={value}
              onChange={(e) => onChange(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="회고 내용을 입력하세요. '/' 키를 누르면 슬래시 커맨드 메뉴(/sprint-stats, /analyse, /issue, /kpt-template)가 나타납니다..."
              className="w-full h-full min-h-[380px] bg-transparent text-slate-900 placeholder-slate-400 resize-none outline-none font-mono text-sm leading-relaxed"
            />

            {showSlashMenu && (
              <SlashMenuPlugin
                onSelect={handleSlashSelect}
                onClose={() => setShowSlashMenu(false)}
                position={menuPos}
              />
            )}
          </>
        ) : (
          <div className="min-h-[380px] p-2">
            {value.trim() === '' ? (
              <p className="text-slate-400 italic text-sm">미리볼 작성 내용이 없습니다.</p>
            ) : (
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={MarkdownComponents as any}
              >
                {value}
              </ReactMarkdown>
            )}
          </div>
        )}
      </div>

      {/* 이슈 선택 팝업 모달 */}
      {showIssueModal && (
        <IssueSelectModal
          onSelect={handleIssueSelect}
          onClose={() => setShowIssueModal(false)}
        />
      )}
    </div>
  );
}
